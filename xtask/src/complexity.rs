use std::{collections::BTreeSet, fs, path::PathBuf, process::Command};

use anyhow::{Context, Result, bail, ensure};
use syn::{
	Attribute, Expr, Meta, Token,
	punctuated::Punctuated,
	spanned::Spanned,
	visit::{self, Visit},
};

type Function = (usize, String, usize);
type Module = (PathBuf, PathBuf);

pub fn run() -> Result<()> {
	let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.unwrap()
		.to_owned();
	let output = Command::new("cargo")
		.args(["metadata", "--no-deps", "--format-version=1", "--locked"])
		.current_dir(&root)
		.output()?;
	ensure!(
		output.status.success(),
		"cargo metadata: {}",
		String::from_utf8_lossy(&output.stderr)
	);
	let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
	let limit = metadata["metadata"]["lint"]["cyclomatic-complexity-threshold"]
		.as_u64()
		.unwrap() as usize;
	let mut pending = targets(&metadata);
	let mut seen = BTreeSet::new();
	let mut reports = Vec::new();
	let mut functions = 0;
	while let Some((path, directory)) = pending.pop() {
		if !seen.insert(path.clone()) {
			continue;
		}
		let source = fs::read_to_string(&path)
			.with_context(|| path.display().to_string())?;
		let syntax = syn::parse_file(&source)
			.with_context(|| path.display().to_string())?;
		let mut check = Check {
			directory,
			pending: &mut pending,
			scores: Vec::new(),
			functions: Vec::new(),
		};
		if !test_only(&syntax.attrs) {
			check.visit_file(&syntax);
		}
		functions += check.functions.len();
		for (line, name, score) in check.functions {
			if score > limit {
				reports.push((
					path.strip_prefix(&root)?.to_owned(),
					line,
					name,
					score,
				));
			}
		}
	}
	reports.sort();
	for (path, line, name, score) in &reports {
		println!(
			"{}:{line}: {name}: cyclomatic complexity {score} > {limit}",
			path.display()
		);
	}
	println!(
		"{} reports / {functions} functions and closures; maximum allowed: {limit}",
		reports.len()
	);
	if !reports.is_empty() {
		bail!("cyclomatic complexity check failed");
	}
	Ok(())
}

fn targets(metadata: &serde_json::Value) -> Vec<Module> {
	let mut pending = Vec::new();
	for package in metadata["packages"].as_array().unwrap() {
		for target in package["targets"].as_array().unwrap() {
			if target["kind"].as_array().unwrap().iter().any(|kind| {
				matches!(kind.as_str(), Some("test" | "bench" | "example"))
			}) {
				continue;
			}
			let path = PathBuf::from(target["src_path"].as_str().unwrap());
			pending.push((path.clone(), path.parent().unwrap().to_owned()));
		}
	}
	pending
}

fn test_only(attrs: &[Attribute]) -> bool {
	attrs.iter().any(|attr| {
		attr.path()
			.segments
			.last()
			.is_some_and(|segment| segment.ident == "test")
			|| (attr.path().is_ident("cfg")
				&& attr
					.parse_args::<Meta>()
					.is_ok_and(|meta| requires_test(&meta)))
	})
}

fn requires_test(meta: &Meta) -> bool {
	match meta {
		Meta::Path(path) => path.is_ident("test"),
		Meta::NameValue(value) if value.path.is_ident("feature") => {
			matches!(&value.value, Expr::Lit(literal) if matches!(&literal.lit, syn::Lit::Str(feature) if feature.value() == "test-support"))
		}
		Meta::List(list)
			if list.path.is_ident("all") || list.path.is_ident("any") =>
		{
			let args = list
				.parse_args_with(
					Punctuated::<Meta, Token![,]>::parse_terminated,
				)
				.unwrap();
			if list.path.is_ident("all") {
				args.iter().any(requires_test)
			} else {
				!args.is_empty() && args.iter().all(requires_test)
			}
		}
		_ => false,
	}
}

struct Check<'a> {
	directory: PathBuf,
	pending: &'a mut Vec<Module>,
	scores: Vec<usize>,
	functions: Vec<Function>,
}

impl Check<'_> {
	fn function(&mut self, name: String, span: proc_macro2::Span, body: &Expr) {
		self.scores.push(1);
		self.visit_expr(body);
		self.functions.push((
			span.start().line,
			name,
			self.scores.pop().unwrap(),
		));
	}
}

impl<'ast> Visit<'ast> for Check<'_> {
	fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
		if test_only(&module.attrs) {
			return;
		}
		let directory = self.directory.join(module.ident.to_string());
		if let Some((_, items)) = &module.content {
			let previous = std::mem::replace(&mut self.directory, directory);
			for item in items {
				self.visit_item(item);
			}
			self.directory = previous;
		} else {
			let explicit = module.attrs.iter().find_map(|attr| {
				if let Meta::NameValue(value) = &attr.meta
					&& value.path.is_ident("path")
					&& let Expr::Lit(literal) = &value.value
					&& let syn::Lit::Str(path) = &literal.lit
				{
					return Some(self.directory.join(path.value()));
				}
				None
			});
			let flat = directory.with_extension("rs");
			let path = explicit.clone().unwrap_or_else(|| {
				if flat.exists() {
					flat
				} else {
					directory.join("mod.rs")
				}
			});
			let children = if explicit.is_some() {
				path.parent().unwrap().to_owned()
			} else {
				directory
			};
			self.pending.push((path, children));
		}
	}

	fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
		if !test_only(&item.attrs) {
			self.scores.push(1);
			self.visit_block(&item.block);
			self.functions.push((
				item.sig.ident.span().start().line,
				item.sig.ident.to_string(),
				self.scores.pop().unwrap(),
			));
		}
	}

	fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
		if !test_only(&item.attrs) {
			visit::visit_item_impl(self, item);
		}
	}

	fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
		if !test_only(&item.attrs) {
			visit::visit_item_trait(self, item);
		}
	}

	fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
		if !test_only(&item.attrs) {
			self.scores.push(1);
			self.visit_block(&item.block);
			self.functions.push((
				item.sig.ident.span().start().line,
				item.sig.ident.to_string(),
				self.scores.pop().unwrap(),
			));
		}
	}

	fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
		if !test_only(&item.attrs)
			&& let Some(body) = &item.default
		{
			self.scores.push(1);
			self.visit_block(body);
			self.functions.push((
				item.sig.ident.span().start().line,
				item.sig.ident.to_string(),
				self.scores.pop().unwrap(),
			));
		}
	}

	fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
		self.function("<closure>".into(), closure.span(), &closure.body);
	}

	fn visit_expr(&mut self, expr: &'ast Expr) {
		let branches = match expr {
			Expr::If(_) | Expr::While(_) | Expr::ForLoop(_) | Expr::Try(_) => 1,
			Expr::Binary(binary)
				if matches!(
					binary.op,
					syn::BinOp::And(_) | syn::BinOp::Or(_)
				) =>
			{
				1
			}
			Expr::Match(expr) => {
				expr.arms.len().saturating_sub(1)
					+ expr.arms.iter().filter(|arm| arm.guard.is_some()).count()
			}
			_ => 0,
		};
		if let Some(score) = self.scores.last_mut() {
			*score += branches;
		}
		visit::visit_expr(self, expr);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn check(source: &str) -> (Vec<Function>, Vec<Module>) {
		let mut pending = Vec::new();
		let mut check = Check {
			directory: PathBuf::from("src"),
			pending: &mut pending,
			scores: Vec::new(),
			functions: Vec::new(),
		};
		check.visit_file(&syn::parse_file(source).unwrap());
		(check.functions, pending)
	}

	#[test]
	fn counts_decisions_and_separates_nested_functions() {
		let (functions, _) = check(
			r#"
			fn outer() {
				if a && b || c {}
				while a {}
				for x in xs {}
				value?;
				match x { 0 => (), 1 if a => (), _ => () }
				let f = || { if a {} };
				fn inner() { if a {} }
			}
			impl T { fn method() { if a {} } }
			trait T { fn default() { if a {} } }
		"#,
		);
		assert_eq!(
			functions
				.iter()
				.map(|(_, name, score)| (name.as_str(), *score))
				.collect::<Vec<_>>(),
			[
				("<closure>", 2),
				("inner", 2),
				("outer", 10),
				("method", 2),
				("default", 2)
			]
		);
	}

	#[test]
	fn excludes_test_modules_and_helpers_but_keeps_production_cfg() {
		let (functions, pending) = check(
			r#"
			#[cfg(test)] mod tests { fn helper() { if a {} } }
			#[cfg(all(test, unix))] mod external_tests;
			#[cfg(test)] fn helper() {}
			#[test] fn regression() {}
			#[tokio::test] async fn async_regression() {}
			#[cfg(feature = "test-support")] mod support;
			#[cfg(test)] impl T { fn helper() {} }
			#[cfg(not(test))] fn production() {}
			#[cfg(any(test, unix))] mod platform;
			mod inline { #[path = "custom.rs"] mod child; }
		"#,
		);
		assert_eq!(
			functions
				.iter()
				.map(|(_, name, _)| name.as_str())
				.collect::<Vec<_>>(),
			["production"]
		);
		assert_eq!(
			pending,
			[
				(
					PathBuf::from("src/platform/mod.rs"),
					PathBuf::from("src/platform")
				),
				(
					PathBuf::from("src/inline/custom.rs"),
					PathBuf::from("src/inline")
				)
			]
		);
	}

	#[test]
	fn accepts_fifteen_and_reports_sixteen() {
		let (functions, _) = check(&format!(
			"fn allowed() {{ {} }} fn exceeded() {{ {} }}",
			"if a {}".repeat(14),
			"if a {}".repeat(15)
		));
		assert_eq!(functions[0].2, 15);
		assert_eq!(functions[1].2, 16);
	}
}
