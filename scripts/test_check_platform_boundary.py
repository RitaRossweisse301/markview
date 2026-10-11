import unittest
from check_platform_boundary import violations


class PlatformBoundaryTests(unittest.TestCase):
    def test_platform_conditions_include_nested_and_multiline_attributes(self):
        for source in ('#[cfg(unix)]', 'cfg!(windows)', '#[cfg(all(test,\n target_os = "android"))]', '#[cfg_attr(not(windows), allow(dead_code))]'):
            self.assertTrue(list(violations(source)))

    def test_only_tests_debugging_and_final_link_attribute_are_allowed(self):
        source = '#[cfg(test)]\n#[cfg(not(test))]\n#[cfg(debug_assertions)]\n#[cfg(all(test, not(debug_assertions)))]'
        self.assertEqual(list(violations(source)), [])
        link = '#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]'
        self.assertEqual(list(violations(link, main=True)), [])
        self.assertTrue(list(violations(link)))


if __name__ == "__main__":
    unittest.main()
