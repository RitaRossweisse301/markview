// The regression host emits pinned test faces as local assets.
import serif from "../../../crates/markview-layout/tests/fonts/NotoSerif-Regular-subset.otf";
import serifBold from "../../../crates/markview-layout/tests/fonts/NotoSerif-Bold-subset.otf";
import serifItalic from "../../../crates/markview-layout/tests/fonts/NotoSerif-Italic-subset.otf";
import sans from "../../../crates/markview-layout/tests/fonts/NotoSans-Regular-subset.otf";
import sansBold from "../../../crates/markview-layout/tests/fonts/NotoSans-Bold-subset.otf";
import sansItalic from "../../../crates/markview-layout/tests/fonts/NotoSans-Italic-subset.otf";
import mono from "../../../crates/markview-layout/tests/fonts/NotoSansMono-Regular-subset.otf";
import monoBold from "../../../crates/markview-layout/tests/fonts/NotoSansMono-Bold-subset.otf";
import cjkSerif from "../../../crates/markview-layout/tests/fonts/NotoSerifCJKsc-Regular-subset.otf";
import cjkSerifBold from "../../../crates/markview-layout/tests/fonts/NotoSerifCJKsc-Bold-subset.otf";
import cjkSans from "../../../crates/markview-layout/tests/fonts/NotoSansCJKsc-Regular-subset.otf";
import cjkSansMedium from "../../../crates/markview-layout/tests/fonts/NotoSansCJKsc-Medium-subset.otf";
import cjkSansBold from "../../../crates/markview-layout/tests/fonts/NotoSansCJKsc-Bold-subset.otf";
import cjkMono from "../../../crates/markview-layout/tests/fonts/NotoSansMonoCJKsc-Regular-subset.otf";
import cjkMonoBold from "../../../crates/markview-layout/tests/fonts/NotoSansMonoCJKsc-Bold-subset.otf";
import emoji from "../../../crates/markview-layout/tests/fonts/NotoColorEmoji-subset.ttf";

export const fonts = [
	serif,
	serifBold,
	serifItalic,
	sans,
	sansBold,
	sansItalic,
	mono,
	monoBold,
	cjkSerif,
	cjkSerifBold,
	cjkSans,
	cjkSansMedium,
	cjkSansBold,
	cjkMono,
	cjkMonoBold,
	emoji,
].map((url) => new URL(url, import.meta.url));
