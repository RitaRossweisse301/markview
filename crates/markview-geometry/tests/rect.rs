use markview_geometry::Rect;

#[test]
fn intersection_requires_positive_area() {
	let rect = Rect {
		x: 10.0,
		y: 20.0,
		w: 30.0,
		h: 40.0,
	};
	let overlap = Rect {
		x: 20.0,
		y: 10.0,
		w: 40.0,
		h: 30.0,
	};
	let expected = Some(Rect {
		x: 20.0,
		y: 20.0,
		w: 20.0,
		h: 20.0,
	});
	assert_eq!(rect.intersect(overlap), expected);
	assert_eq!(overlap.intersect(rect), expected);
	assert_eq!(rect.intersect(rect), Some(rect));
	for x in [40.0, 50.0] {
		assert_eq!(rect.intersect(Rect { x, ..rect }), None);
	}
	for w in [0.0, -1.0] {
		assert_eq!(rect.intersect(Rect { w, ..rect }), None);
	}
}

#[test]
fn containment_includes_edges_but_excludes_empty_rectangles() {
	let rect = Rect {
		x: 10.0,
		y: 20.0,
		w: 30.0,
		h: 40.0,
	};
	for (x, y) in [(10.0, 20.0), (40.0, 60.0), (25.0, 40.0)] {
		assert!(rect.contains(x, y));
	}
	for (x, y) in [(9.0, 40.0), (41.0, 40.0), (25.0, 19.0), (25.0, 61.0)] {
		assert!(!rect.contains(x, y));
	}
	for extent in [0.0, -1.0] {
		assert!(!Rect { w: extent, ..rect }.contains(10.0, 20.0));
		assert!(!Rect { h: extent, ..rect }.contains(10.0, 20.0));
	}
}
