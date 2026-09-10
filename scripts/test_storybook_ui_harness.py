#!/usr/bin/env python3
import tempfile
import unittest
import importlib.util
from pathlib import Path

from storybook_ui_harness_sources import StorybookUiHarnessSources

MODULE_PATH = Path(__file__).with_name("assert-storybook-ui-harness.py")
SPEC = importlib.util.spec_from_file_location("assert_storybook_ui_harness", MODULE_PATH)
assert SPEC is not None
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)
StorybookUiHarness = MODULE.StorybookUiHarness

MANIFEST_MODULE_PATH = Path(__file__).with_name("storybook_ui_harness_manifest.py")
MANIFEST_SPEC = importlib.util.spec_from_file_location(
    "storybook_ui_harness_manifest", MANIFEST_MODULE_PATH
)
assert MANIFEST_SPEC is not None
MANIFEST_MODULE = importlib.util.module_from_spec(MANIFEST_SPEC)
assert MANIFEST_SPEC.loader is not None
MANIFEST_SPEC.loader.exec_module(MANIFEST_MODULE)
StorybookUiHarnessManifest = MANIFEST_MODULE.StorybookUiHarnessManifest


def write_text(path: Path, source: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(source, encoding="utf-8")


class StorybookUiHarnessTest(unittest.TestCase):
    def test_runtime_pages_are_parsed_separately_from_canvas_requirements(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_text(
                root / "crates/katana-ui-core-storybook/src/requirements.rs",
                'const CANVAS_REQUIRED_PAGES: &[&str] = &["text", "button"];\n'
                'const INTERACTIVE_RUNTIME_PAGES: &[&str] = &["command-chrome"];\n',
            )

            sources = StorybookUiHarnessSources(root)

            self.assertEqual(["text", "button"], sources.required_pages())
            self.assertEqual(["command-chrome"], sources.interactive_runtime_pages())

    def test_rejects_runtime_page_without_eframe_dispatch_or_shared_surface(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_runtime_page_fixture(root)

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "command-chrome: runtime contract missing `command_chrome_runtime::open_window` "
                "in crates/katana-ui-core-storybook/src/visual/window.rs",
                failures,
            )
            self.assertIn(
                "command-chrome: runtime contract missing `EguiCommandChromeAdapter` "
                "in crates/katana-ui-core-storybook/src/visual/command_chrome_surface.rs",
                failures,
            )

    def test_accepts_text_command_root_with_strict_facade_artifact_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_text_command_root_runtime_fixture(root)

            self.assertEqual([], StorybookUiHarness(root).failures())

    def test_rejects_text_command_root_without_facade_mp4_or_manifest_evidence(self) -> None:
        required_tokens = (
            (
                "EguiTextCommandSurfaceHostProjectionEncoder::token",
                "crates/katana-ui-core-storybook/src/visual/text_command_root_storybook.rs",
            ),
            (
                '"framemd5"',
                "crates/katana-ui-core-storybook/src/visual/text_command_root_storybook.rs",
            ),
            (
                '"text-command-root-manifest.json"',
                "crates/katana-ui-core-storybook/src/visual/text_command_root_storybook.rs",
            ),
            (
                "compatibility_types_are_hidden",
                "crates/katana-ui-core/tests/egui_host_root_facade_contract.rs",
            ),
            (
                "full_root_storybook_uses_only_the_public_facade_root",
                "crates/katana-ui-core-storybook/src/visual/text_command_root_storybook_tests.rs",
            ),
        )
        for missing_token, expected_path in required_tokens:
            with self.subTest(missing_token=missing_token), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
                write_text_command_root_runtime_fixture(root, missing_token=missing_token)

                failures = StorybookUiHarness(root).failures()

                self.assertIn(
                    "text-command-root: runtime contract missing "
                    f"`{missing_token}` in {expected_path}",
                    failures,
                )

    def test_rejects_runtime_page_in_canvas_menu_preset_or_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_runtime_page_fixture(root, complete=True)
            write_text(
                root / "crates/katana-ui-core-storybook/src/catalog/story_paths_runtime.rs",
                'StoryPath { page: "command-chrome" }\n',
            )
            write_text(
                root / "crates/katana-ui-core-storybook/src/catalog/preset_labels.rs",
                '"command-chrome" => &["a", "b", "c", "d"],\n',
            )
            write_minimal_manifest(root, pages=("text", "button", "command-chrome"))

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "command-chrome: interactive runtime page must not appear in the Canvas menu",
                failures,
            )
            self.assertIn(
                "command-chrome: interactive runtime page must not use Canvas preset labels",
                failures,
            )
            self.assertIn(
                "command-chrome: interactive runtime page must not appear in the Canvas manifest",
                failures,
            )

    def test_accepts_required_pages_with_presets_options_and_inspector_route(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(
                root,
                option_arm=(
                    '"button" | "text-button" | "svg-button" | "icon-text-button" '
                    "=> &BUTTON_OPTIONS,"
                ),
            )

            self.assertEqual([], StorybookUiHarness(root).failures())

    def test_rejects_missing_option_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            source = option_contract_source('"button" => &BUTTON_OPTIONS,')
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/storybook_ui_option_contract.rs",
                source.replace('"text" => &TEXT_OPTIONS,', ""),
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn("text: missing Storybook UI option contract", failures)

    def test_rejects_manifest_missing_required_page(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_minimal_manifest(root, pages=("text",))

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "docs/storybook-77ui-interaction-manifest.json: "
                "required page missing from manifest: button",
                failures,
            )

    def test_rejects_manifest_test_placeholder_reference(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_minimal_manifest(root, pages=("text", "button"))
            manifest = root / "docs/storybook-77ui-interaction-manifest.json"
            source = manifest.read_text(encoding="utf-8")
            manifest.write_text(
                source.replace(
                    "shared:visual_interaction_tests",
                    "visual/visual_interaction_<page>_tests.rs",
                    1,
                ),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "docs/storybook-77ui-interaction-manifest.json: "
                "text.tests.visual_interaction must not use <page> placeholders",
                failures,
            )

    def test_rejects_manifest_missing_test_reference(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_minimal_manifest(root, pages=("text", "button"))
            manifest = root / "docs/storybook-77ui-interaction-manifest.json"
            source = manifest.read_text(encoding="utf-8")
            manifest.write_text(
                source.replace("shared:guard", "scripts/missing-guard.py", 1),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "docs/storybook-77ui-interaction-manifest.json: "
                "text.tests.guard reference does not exist: scripts/missing-guard.py",
                failures,
            )

    def test_rejects_manual_pending_manifest_without_manual_acceptance_smoke_guard(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_text(root / "scripts/assert-storybook-ui-harness.py", "# fixture\n")
            write_text(root / "scripts/storybook_ui_harness_manifest.py", "# fixture\n")
            write_text(root / "scripts/storybook_manual_acceptance_smoke.py", "# fixture\n")
            write_text(root / "scripts/test_storybook_manual_acceptance_smoke.py", "# fixture\n")
            write_text(
                root / "docs/storybook-77ui-interaction-manifest.json",
                "{"
                '"schema_version":1,'
                '"policy":{'
                '"harness_scope":"core_public_api_harness",'
                '"uses_storybook_only_state":false,'
                '"uses_inspector_only_change":false,'
                '"uses_preset_label_only_change":false,'
                '"bypasses_core_public_api":false'
                "},"
                '"defaults_by_engine":{"layout_alignment":{'
                '"public_props_options":["props"],'
                '"state":["state"],'
                '"action":["action"],'
                '"event":["event"],'
                '"callback":["callback"],'
                '"required_operations":["pointer"],'
                '"tests":{'
                '"window_interaction":["shared:window"],'
                '"visual_interaction":["shared:visual"],'
                '"guard":["scripts/assert-storybook-ui-harness.py","scripts/storybook_ui_harness_manifest.py"]'
                "}}},"
                '"ui":[{'
                '"page":"progress-bar",'
                '"engine":"layout_alignment",'
                '"audit_status":"partial",'
                '"evidence":['
                '"progress_bar_timed_tick_advances_via_core_progress_action",'
                '"progress_bar_timed_tick_cycles_after_reaching_maximum",'
                '"progress_bar_live_audit_reports_timed_tick_progress_contract",'
                '"progress_bar_live_audit_reports_timed_cycle_after_maximum",'
                '"progress_bar_live_audit_reports_indeterminate_segment_motion",'
                '"progress_bar_indeterminate_segment_moves_on_runtime_tick",'
                '"progress_bar_dedicated_render_uses_core_progress_bar_public_api",'
                '"progress_bar_window_runtime_tick_repaints_meter_body",'
                '"progress_bar_window_runtime_tick_cycles_after_maximum"'
                "],"
                '"gaps":["manual_acceptance_pending: user confirmation is required"]'
                "}]"
                "}\n",
            )

            failures = StorybookUiHarnessManifest(root, ["progress-bar"]).failures()

            self.assertIn(
                "docs/storybook-77ui-interaction-manifest.json: "
                "progress-bar manual pending tests.guard must include "
                "scripts/storybook_manual_acceptance_smoke.py",
                failures,
            )

    def test_rejects_manual_pending_manifest_without_acceptance_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_text(root / "scripts/assert-storybook-ui-harness.py", "# fixture\n")
            write_text(root / "scripts/storybook_ui_harness_manifest.py", "# fixture\n")
            write_text(root / "scripts/storybook_manual_acceptance_smoke.py", "# fixture\n")
            write_text(root / "scripts/test_storybook_manual_acceptance_smoke.py", "# fixture\n")
            write_text(
                root / "docs/storybook-77ui-interaction-manifest.json",
                "{"
                '"schema_version":1,'
                '"policy":{'
                '"harness_scope":"core_public_api_harness",'
                '"uses_storybook_only_state":false,'
                '"uses_inspector_only_change":false,'
                '"uses_preset_label_only_change":false,'
                '"bypasses_core_public_api":false'
                "},"
                '"ui":[{'
                '"page":"progress-bar",'
                '"group":"atoms",'
                '"engine":"layout_alignment",'
                '"public_props_options":["props"],'
                '"state":["state"],'
                '"action":["action"],'
                '"event":["event"],'
                '"callback":["callback"],'
                '"required_operations":["pointer","timed_tick"],'
                '"minimum_observation_frames":48,'
                '"tests":{'
                '"window_interaction":["shared:window"],'
                '"visual_interaction":["shared:visual"],'
                '"guard":["scripts/storybook_manual_acceptance_smoke.py","scripts/test_storybook_manual_acceptance_smoke.py"]'
                "},"
                '"audit_status":"partial",'
                '"evidence":['
                '"progress_bar_timed_tick_advances_via_core_progress_action",'
                '"progress_bar_timed_tick_cycles_after_reaching_maximum",'
                '"progress_bar_live_audit_reports_timed_tick_progress_contract",'
                '"progress_bar_live_audit_reports_timed_cycle_after_maximum",'
                '"progress_bar_live_audit_reports_indeterminate_segment_motion",'
                '"progress_bar_indeterminate_segment_moves_on_runtime_tick",'
                '"progress_bar_dedicated_render_uses_core_progress_bar_public_api",'
                '"progress_bar_window_runtime_tick_repaints_meter_body",'
                '"progress_bar_window_runtime_tick_cycles_after_maximum"'
                "],"
                '"acceptance_checks":["progress_timed_tick","progress_timed_cycle"],'
                '"acceptance_observations":["meter advances from 65% to 82%"],'
                '"gaps":["manual_acceptance_pending: user confirmation is required"]'
                "}]"
                "}\n",
            )
            manifest = root / "docs/storybook-77ui-interaction-manifest.json"
            source = manifest.read_text(encoding="utf-8")
            source = source.replace(
                '"acceptance_checks":["progress_timed_tick","progress_timed_cycle"],',
                "",
            )
            source = source.replace(
                '"acceptance_observations":["meter advances from 65% to 82%"],',
                "",
            )
            source = source.replace('"minimum_observation_frames":48,', "")
            manifest.write_text(source, encoding="utf-8")

            failures = StorybookUiHarnessManifest(root, ["progress-bar"]).failures()

            self.assertIn(
                "docs/storybook-77ui-interaction-manifest.json: "
                "progress-bar.acceptance_checks must be a non-empty array while manual acceptance is pending",
                failures,
            )
            self.assertIn(
                "docs/storybook-77ui-interaction-manifest.json: "
                "progress-bar.acceptance_observations must be a non-empty array while manual acceptance is pending",
                failures,
            )
            self.assertIn(
                "docs/storybook-77ui-interaction-manifest.json: "
                "progress-bar.minimum_observation_frames must be a positive integer while manual acceptance is pending",
                failures,
            )

    def test_rejects_manual_acceptance_smoke_without_fresh_live_audit(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            justfile = root / "Justfile"
            justfile.write_text(
                justfile.read_text(encoding="utf-8").replace(
                    "    cargo run --release -p katana-ui-core-storybook --bin "
                    "katana-ui-core-storybook --locked -- --headless-interaction-audit\n",
                    "",
                ),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "Justfile: storybook-manual-acceptance-smoke must regenerate live interaction audit",
                failures,
            )

    def test_rejects_text_manual_pending_without_selection_copy_checks(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_text(root / "scripts/assert-storybook-ui-harness.py", "# fixture\n")
            write_text(root / "scripts/storybook_ui_harness_manifest.py", "# fixture\n")
            write_text(root / "scripts/storybook_manual_acceptance_queue.py", "# fixture\n")
            write_text(root / "scripts/test_storybook_manual_acceptance_queue.py", "# fixture\n")
            write_text(
                root / "scripts/storybook_manual_acceptance_smoke.py",
                "storybook-manual-acceptance-evidence.json\n"
                "manual_acceptance_evidence_report\n"
                "write_evidence_report\n"
                '"command": entry.get("command")\n'
                '"smoke_command": entry.get("smoke_command")\n'
                '"minimum_observation_frames": entry.get("minimum_observation_frames")\n',
            )
            write_text(root / "scripts/test_storybook_manual_acceptance_smoke.py", "# fixture\n")
            write_text(
                root / "docs/storybook-77ui-interaction-manifest.json",
                "{"
                '"schema_version":1,'
                '"ui":[{'
                '"page":"text",'
                '"audit_status":"partial",'
                '"manual_acceptance_order":10,'
                '"dependency_layer":"foundation-text-selection",'
                '"depends_on":[],'
                '"required_operations":["drag","keyboard"],'
                '"minimum_observation_frames":1,'
                '"acceptance_checks":[],'
                '"acceptance_observations":["manual"],'
                '"gaps":["manual_acceptance_pending: user confirmation is required"]'
                "}]"
                "}\n",
            )
            write_text(
                root / "Justfile",
                "storybook-manual-acceptance-smoke:\n"
                "    PYTHONPATH=scripts python3 scripts/storybook_manual_acceptance_smoke.py\n"
                "    cargo run -- --headless-interaction-audit\n"
                "storybook-regression: storybook-manual-acceptance-smoke\n",
            )

            failures = StorybookUiHarness(root).manual_acceptance_queue_failures()

            self.assertIn(
                "text: acceptance_checks must be a non-empty string array",
                failures,
            )

    def test_rejects_manual_acceptance_smoke_without_evidence_report(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            smoke = root / "scripts/storybook_manual_acceptance_smoke.py"
            smoke.write_text(
                smoke.read_text(encoding="utf-8").replace(
                    "manual_acceptance_evidence_report",
                    "manual_acceptance_label_only_report",
                ),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "manual acceptance smoke must write storybook-manual-acceptance-evidence.json",
                failures,
            )

    def test_rejects_manual_acceptance_smoke_without_replay_command_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            smoke = root / "scripts/storybook_manual_acceptance_smoke.py"
            smoke.write_text(
                smoke.read_text(encoding="utf-8").replace(
                    '"smoke_command": entry.get("smoke_command"),',
                    "",
                ),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "manual acceptance smoke must write storybook-manual-acceptance-evidence.json",
                failures,
            )

    def test_rejects_verified_progress_bar_without_tick_and_core_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_progress_manifest(root, evidence=("old progress evidence",))

            failures = StorybookUiHarnessManifest(root, ["progress-bar"]).failures()

            self.assertIn(
                "docs/storybook-77ui-interaction-manifest.json: "
                "progress-bar evidence must include "
                "progress_bar_timed_tick_advances_via_core_progress_action",
                failures,
            )
            self.assertIn(
                "docs/storybook-77ui-interaction-manifest.json: "
                "progress-bar evidence must include "
                "progress_bar_dedicated_render_uses_core_progress_bar_public_api",
                failures,
            )
            self.assertIn(
                "docs/storybook-77ui-interaction-manifest.json: "
                "progress-bar evidence must include "
                "progress_bar_live_audit_reports_indeterminate_segment_motion",
                failures,
            )
            self.assertIn(
                "docs/storybook-77ui-interaction-manifest.json: "
                "progress-bar evidence must include "
                "progress_bar_indeterminate_segment_moves_on_runtime_tick",
                failures,
            )

    def test_rejects_low_option_count(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = root / "crates/katana-ui-core-storybook/src/visual/storybook_ui_option_contract.rs"
            source = path.read_text(encoding="utf-8")
            path.write_text(source.replace("new(\"d\", \"0\", \"1\"),", ""), encoding="utf-8")

            failures = StorybookUiHarness(root).failures()

            self.assertIn("text: Storybook option contract must cover at least 4 options", failures)

    def test_accepts_option_counts_from_split_source_file(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = root / "crates/katana-ui-core-storybook/src/visual/storybook_ui_option_contract.rs"
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.split("const BUTTON_OPTIONS", 1)[0],
                encoding="utf-8",
            )
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/storybook_ui_runtime_options.rs",
                "const BUTTON_OPTIONS: [StorybookUiOptionContract; 4] = ["
                'StorybookUiOptionContract::new("a", "0", "1"),'
                'StorybookUiOptionContract::new("b", "0", "1"),'
                'StorybookUiOptionContract::new("c", "0", "1"),'
                'StorybookUiOptionContract::new("d", "0", "1"),];\n',
            )

            self.assertEqual([], StorybookUiHarness(root).failures())

    def test_accepts_multiline_storybook_option_contract_setting(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/storybook_ui_runtime_options.rs",
                "const BUTTON_OPTIONS: [StorybookUiOptionContract; 4] = ["
                "StorybookUiOptionContract::new(\n"
                '    "a",\n'
                '    "0",\n'
                '    "1",\n'
                "),"
                'StorybookUiOptionContract::new("b", "0", "1"),'
                'StorybookUiOptionContract::new("c", "0", "1"),'
                'StorybookUiOptionContract::new("d", "0", "1"),];\n',
            )

            failures = StorybookUiHarness(root).failures()

            self.assertNotIn(
                "button: Storybook option contract must cover at least 4 options",
                failures,
            )

    def test_rejects_low_preset_count(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = root / "crates/katana-ui-core-storybook/src/catalog/preset_labels.rs"
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace('"button" => &["a","b","c","d"]', '"button" => &["a","b","c"]'),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn("button: Storybook presets must expose at least 4 tabs", failures)

    def test_rejects_missing_leaf_change(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            leaf = root / "openspec/changes/storybook-page-text/proposal.md"
            leaf.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "text: leaf change `storybook-page-text` missing openspec/changes/storybook-page-text/proposal.md",
                failures,
            )

    def test_rejects_missing_priority_number(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_text(
                root
                / "openspec/changes/establish-kuc-atoms-molecules-catalog/storybook-menu-priority-order.md",
                "| priority | menu page | leaf change | reason |\n"
                "| --- | --- | --- | --- |\n"
                "| SB-001 | `text` | `storybook-page-text` | test |\n",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn("button: Storybook menu page missing priority number", failures)

    def test_rejects_missing_dedicated_draw_branch(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_text(
                root / "crates/katana-ui-core-storybook/src/visual/dedicated.rs",
                'fn draw_page(page: &str) { match page { "text" => text(), _ => draw() } }\n',
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "button: split table says page-specific rendering exists, but draw_page has no branch",
                failures,
            )

    def test_rejects_stale_split_summary_counts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = root / "openspec/changes/establish-kuc-atoms-molecules-catalog/storybook-menu-change-split.md"
            source = path.read_text(encoding="utf-8")
            path.write_text(source.replace("page 別描画あり: 2", "page 別描画あり: 1"), encoding="utf-8")

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "storybook menu split summary count `あり` must be 2, got 1",
                failures,
            )

    def test_rejects_missing_window_interaction_required_page_tests(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = root / "crates/katana-ui-core-storybook/src/visual/window_interaction/tests/required_page_tests.rs"
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace("component_body_pixel_diff", "component_body_delta"),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "window_interaction required-page tests missing token: component_body_pixel_diff",
                failures,
            )

    def test_rejects_layout_pages_without_specific_window_interaction_tests(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_text(
                root / "crates/katana-ui-core-storybook/src/requirements.rs",
                'const REQUIRED_PAGES: &[&str] = &["text", "button", "align-center", "scroll-area"];\n'
                "const MIN_SINGLE_NODE: usize = 1;\n",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "align-center: missing specific window_interaction test token: "
                "align_center_window_interaction_click_updates_preview_state",
                failures,
            )
            self.assertIn(
                "scroll-area: missing specific window_interaction test token: "
                "scroll_area_window_interaction_scroll_updates_preview_state",
                failures,
            )

    def test_rejects_missing_inspector_option_contract_test(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = root / "crates/katana-ui-core-storybook/src/visual/visual_inspector_option_contract_tests.rs"
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("Inspector option contract test is missing", failures)

    def test_rejects_inspector_option_contract_without_click_application(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = root / "crates/katana-ui-core-storybook/src/visual/visual_inspector_option_contract_tests.rs"
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace("apply_click(&mut state;", "click_state();"),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "Inspector option contract test missing token: apply_click(&mut state",
                failures,
            )

    def test_rejects_missing_inspector_fallback_status_test(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_inspector_fallback_status_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("Inspector fallback status test is missing", failures)

    def test_rejects_missing_preset_distinct_contract_for_required_page(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = root / "crates/katana-ui-core-storybook/src/visual/visual_text_tests.rs"
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "text: preset tabs must have a distinct rendering contract test",
                failures,
            )

    def test_rejects_preset_tabs_that_do_not_cover_every_option_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/storybook_ui_option_contract.rs"
            )
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace(
                    "const BUTTON_OPTIONS: [StorybookUiOptionContract; 4] = ["
                    'StorybookUiOptionContract::new("a", "0", "1"),'
                    'StorybookUiOptionContract::new("b", "0", "1"),'
                    'StorybookUiOptionContract::new("c", "0", "1"),'
                    'StorybookUiOptionContract::new("d", "0", "1"),];',
                    "const BUTTON_OPTIONS: [StorybookUiOptionContract; 5] = ["
                    'StorybookUiOptionContract::new("a", "0", "1"),'
                    'StorybookUiOptionContract::new("b", "0", "1"),'
                    'StorybookUiOptionContract::new("c", "0", "1"),'
                    'StorybookUiOptionContract::new("d", "0", "1"),'
                    'StorybookUiOptionContract::new("e", "0", "1"),];',
                ),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "button: Storybook preset tabs must cover every option contract "
                "(4 presets < 5 options)",
                failures,
            )

    def test_rejects_public_option_without_storybook_inspector_option(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_text(
                root / "crates/katana-ui-core/src/atom/typed.rs",
                "impl Input { pub fn clear_action(mut self) -> Self { self } }\n",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "text-input: public option `pub fn clear_action` "
                "missing Storybook Inspector option `text_entry.clear_action`",
                failures,
            )

    def test_accepts_public_option_when_storybook_inspector_option_exists(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_text(
                root / "crates/katana-ui-core/src/atom/typed.rs",
                "impl Input { pub fn clear_action(mut self) -> Self { self } }\n",
            )
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/storybook_ui_form_options.rs",
                "const INPUT_OPTIONS: [StorybookUiOptionContract; 1] = ["
                'StorybookUiOptionContract::new("text_entry.clear_action", "none", "visible"),'
                "];\n",
            )
            path = root / "crates/katana-ui-core-storybook/src/visual/storybook_ui_option_contract.rs"
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace(
                    '"button" => &BUTTON_OPTIONS,',
                    '"button" => &BUTTON_OPTIONS, "text-input" => &INPUT_OPTIONS,',
                ),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertNotIn(
                "text-input: public option `pub fn clear_action` "
                "missing Storybook Inspector option `text_entry.clear_action`",
                failures,
            )

    def test_rejects_button_common_public_option_without_storybook_inspector_option(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_text(
                root / "crates/katana-ui-core/src/atom/mod.rs",
                "impl Button { pub fn focusable(mut self) -> Self { self } }\n",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "button: public option `pub fn focusable` "
                "missing Storybook Inspector option `focusable`",
                failures,
            )

    def test_accepts_button_common_public_option_when_storybook_inspector_option_exists(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(
                root,
                option_arm=(
                    '"button" | "text-button" | "svg-button" | "icon-text-button" '
                    "=> &BUTTON_OPTIONS,"
                ),
            )
            write_text(
                root / "crates/katana-ui-core/src/atom/mod.rs",
                "impl Button { pub fn focusable(mut self) -> Self { self } }\n",
            )
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/storybook_ui_runtime_options.rs",
                "const BUTTON_OPTIONS: [StorybookUiOptionContract; 4] = ["
                'StorybookUiOptionContract::new("focusable", "true", "false"),'
                'StorybookUiOptionContract::new("b", "0", "1"),'
                'StorybookUiOptionContract::new("c", "0", "1"),'
                'StorybookUiOptionContract::new("d", "0", "1"),];\n',
            )

            failures = StorybookUiHarness(root).failures()

            self.assertNotIn(
                "button: public option `pub fn focusable` "
                "missing Storybook Inspector option `focusable`",
                failures,
            )

    def test_rejects_screen_state_store_without_page_preset_key_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = root / "crates/katana-ui-core-storybook/src/visual/window_interaction/state_store.rs"
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace("preset_index: usize", "index: usize"),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "Storybook screen state store contract missing token: preset_index: usize",
                failures,
            )

    def test_rejects_screen_state_store_without_instance_key_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = root / "crates/katana-ui-core-storybook/src/visual/window_interaction/state_store.rs"
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace("instance_id: &'static str", "instance: &'static str"),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "Storybook screen state store contract missing token: instance_id: &'static str",
                failures,
            )

    def test_rejects_screen_state_store_without_required_page_instance_tests(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/window_interaction/state_store_tests.rs"
            )
            path.write_text("", encoding="utf-8")

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "Storybook screen state store instance test missing token: "
                "every_required_page_keeps_screen_state_instances_separate",
                failures,
            )

    def test_rejects_text_input_singleton_runtime_state(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_text(
                root / "crates/katana-ui-core-storybook/src/visual/screen_state.rs",
                "struct StorybookScreenState { text_input_state: UiComponentState }\n",
            )
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/text_input_screen_state.rs",
                "struct TextInputStateStore { instances: BTreeMap<&'static str, TextInputRuntimeState> }\n",
            )
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/screen_state_text_input.rs",
                "fn register_text_input_readonly_block() {}\n",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "text-input Storybook state must be instance-scoped, not `text_input_state:`",
                failures,
            )

    def test_rejects_text_area_singleton_runtime_state(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_text(
                root / "crates/katana-ui-core-storybook/src/visual/screen_state.rs",
                "struct StorybookScreenState { text_area_value: String }\n",
            )
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/text_area_screen_state.rs",
                "struct TextAreaStateStore { instances: BTreeMap<&'static str, TextAreaRuntimeState> }\n"
                'const DEFAULT_TEXT_AREA_INSTANCE: &str = "text-area.preview";\n',
            )
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/screen_state_text_area.rs",
                "fn text_area_runtime_for(instance) {}\nfn text_area_runtime_mut_for(instance) {}\n",
            )
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/screen_state_text_area_scroll.rs",
                "fn text_area_runtime_mut_for(instance) {}\n",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "text-area Storybook state must be instance-scoped, not `text_area_value:`",
                failures,
            )

    def test_accepts_text_area_runtime_read_from_query_split(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/text_area_screen_state.rs",
                "struct TextAreaStateStore { instances: BTreeMap<&'static str, TextAreaRuntimeState> }\n"
                'const DEFAULT_TEXT_AREA_INSTANCE: &str = "text-area.preview";\n',
            )
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/screen_state_text_area.rs",
                "fn text_area_runtime_mut_for(instance) {}\n",
            )
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/screen_state_text_area_scroll.rs",
                "fn text_area_runtime_mut_for(instance) {}\n",
            )
            write_text(
                root
                / "crates/katana-ui-core-storybook/src/visual/screen_state_text_area_queries.rs",
                "fn text_area_runtime_for(instance) {}\n",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertNotIn(
                "text-area Storybook preview state must read runtime store",
                failures,
            )

    def test_rejects_preset_tab_labels_without_measured_clip(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            write_text(
                root / "crates/katana-ui-core-storybook/src/visual/preset_tabs.rs",
                "fn draw_tab_label() {}\n",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "preset tab labels must be measured and clipped inside each tab: missing preset_tab_label::fit",
                failures,
            )

    def test_rejects_button_family_contract_without_menu_button_cursor(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/window_interaction/tests/button_operation_tests.rs"
            )
            source = path.read_text(encoding="utf-8")
            path.write_text(source.replace('"menu-button"', '"menu"'), encoding="utf-8")

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                'button cursor contract missing token: "menu-button"',
                failures,
            )

    def test_rejects_button_family_contract_without_menu_hover_border(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = root / "crates/katana-ui-core-storybook/src/visual/visual_interaction_menu_button_tests.rs"
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace("menu_button_hover_draws_shared_button_family_border_token", ""),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "menu-button hover contract missing token: "
                "menu_button_hover_draws_shared_button_family_border_token",
                failures,
            )

    def test_rejects_button_family_contract_without_measured_center(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = root / "crates/katana-ui-core-storybook/src/visual/visual_interaction_button_center_tests.rs"
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace("measure_button_label_width", "label_chars_count"),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "button measured center contract missing token: measure_button_label_width",
                failures,
            )

    def test_rejects_button_option_contract_without_action_event_state(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_inspector_button_preset_tests.rs"
            )
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace(
                    "button_inspector_rows_apply_action_event_and_state_for_every_button_page",
                    "button_inspector_rows_select_matching_preset_tabs_only",
                ),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "button option state contract missing token: "
                "button_inspector_rows_apply_action_event_and_state_for_every_button_page",
                failures,
            )

    def test_rejects_button_family_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_button_instance_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("button family instance interaction tests are missing", failures)

    def test_rejects_selection_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_selection_instance_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("selection instance interaction tests are missing", failures)

    def test_rejects_selection_list_preset_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_selection_list_preset_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("selection-list preset instance interaction tests are missing", failures)

    def test_rejects_side_menu_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_side_menu_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("side-menu instance interaction tests are missing", failures)

    def test_rejects_tree_view_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_tree_view_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("tree-view instance interaction tests are missing", failures)

    def test_rejects_menu_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_menu_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("menu instance interaction tests are missing", failures)

    def test_rejects_context_menu_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_context_menu_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("context-menu instance interaction tests are missing", failures)

    def test_rejects_color_picker_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_color_picker_options_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("color-picker-rgba instance interaction tests are missing", failures)

    def test_rejects_color_picker_options_without_runtime_state_assertions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_color_picker_options_tests.rs"
            )
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace("assert_color_picker_runtime;", "assert_color_picker_label_only;"),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "color-picker option semantic state test missing token: "
                "assert_color_picker_runtime",
                failures,
            )

    def test_rejects_settings_list_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_settings_list_options_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("settings-list instance interaction tests are missing", failures)

    def test_rejects_settings_list_options_without_runtime_state_assertions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_settings_list_options_tests.rs"
            )
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace("assert_settings_list_runtime;", "assert_settings_list_label_only;"),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "settings-list option semantic state test missing token: "
                "assert_settings_list_runtime",
                failures,
            )

    def test_rejects_diagnostics_list_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_diagnostics_list_options_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("diagnostics-list instance interaction tests are missing", failures)

    def test_rejects_diagnostics_list_options_without_runtime_state_assertions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_diagnostics_list_options_tests.rs"
            )
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace(
                    "assert_diagnostics_list_runtime;",
                    "assert_diagnostics_list_label_only;",
                ),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "diagnostics-list option semantic state test missing token: "
                "assert_diagnostics_list_runtime",
                failures,
            )

    def test_rejects_toolbar_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_toolbar_state_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("toolbar instance interaction tests are missing", failures)

    def test_rejects_text_input_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_text_input_state_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("text-input instance interaction tests are missing", failures)

    def test_rejects_text_area_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_text_area_state_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("text-area instance interaction tests are missing", failures)

    def test_rejects_dynamic_array_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_dynamic_array_editor_state_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "dynamic-array-editor instance interaction tests are missing",
                failures,
            )

    def test_rejects_drag_and_drop_instance_interaction_tests_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_drag_and_drop_state_tests.rs"
            )
            path.unlink()

            failures = StorybookUiHarness(root).failures()

            self.assertIn("drag-and-drop instance interaction tests are missing", failures)

    def test_rejects_command_palette_instance_interaction_test_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_command_palette_options_tests.rs"
            )
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace(
                    "command_palette_window_interaction_keeps_query_and_highlight_instance_isolated",
                    "command_palette_window_interaction_missing",
                ),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "command-palette instance interaction test missing token: "
                "command_palette_window_interaction_keeps_query_and_highlight_instance_isolated",
                failures,
            )

    def test_rejects_command_palette_options_without_runtime_state_assertions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_command_palette_options_tests.rs"
            )
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace(
                    "assert_command_palette_runtime;",
                    "assert_command_palette_label_only;",
                ),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "command-palette option semantic state test missing token: "
                "assert_command_palette_runtime",
                failures,
            )

    def test_rejects_shortcut_cheatsheet_options_without_runtime_state_assertions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_shortcut_cheatsheet_options_tests.rs"
            )
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace(
                    "assert_shortcut_cheatsheet_runtime;",
                    "assert_shortcut_cheatsheet_label_only;",
                ),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "shortcut-cheatsheet option semantic state test missing token: "
                "assert_shortcut_cheatsheet_runtime",
                failures,
            )

    def test_rejects_runtime_structured_options_without_runtime_state_assertions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, option_arm='"button" => &BUTTON_OPTIONS,')
            path = (
                root
                / "crates/katana-ui-core-storybook/src/visual/visual_interaction_runtime_structured_options_tests.rs"
            )
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace(
                    "assert_runtime_structured_state;",
                    "assert_runtime_structured_label_only;",
                ),
                encoding="utf-8",
            )

            failures = StorybookUiHarness(root).failures()

            self.assertIn(
                "runtime structured option semantic state test missing token: "
                "assert_runtime_structured_state",
                failures,
            )


def write_minimal_repo(root: Path, option_arm: str) -> None:
    write_text(
        root / "Justfile",
        "storybook-manual-acceptance-smoke:\n"
        "    cargo run --release -p katana-ui-core-storybook --bin "
        "katana-ui-core-storybook --locked -- --headless-interaction-audit\n"
        "    PYTHONPATH=scripts python3 scripts/storybook_manual_acceptance_smoke.py\n"
        "storybook-regression: cargo-test storybook-check storybook-smoke storybook-manual-acceptance-smoke storybook-interaction-smoke storybook-requirement-gate\n",
    )
    write_text(
        root / "scripts/storybook_manual_acceptance_queue.py",
        "from pathlib import Path\n"
        "def manual_acceptance_queue(manifest: Path):\n"
        "    return [{\n"
        '        "page": "progress-bar",\n'
        '        "command": "rtk cargo run --release -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked -- --open-window progress-bar",\n'
        '        "smoke_command": "rtk cargo run --release -p katana-ui-core-storybook --bin katana-ui-core-storybook --locked -- --open-window 48 progress-bar",\n'
        '        "minimum_observation_frames": 48,\n'
        '        "acceptance_checks": ["progress_preview_click", "progress_timed_tick", "progress_timed_cycle", "progress_indeterminate_segment_motion"],\n'
        '        "acceptance_observations": acceptance_observations("progress-bar"),\n'
        "    }]\n"
        "def acceptance_observations(page: str):\n"
        "    return ['meter advances from 65% to 82%', 'meter cycles back to 0% after max']\n",
    )
    write_text(root / "scripts/test_storybook_manual_acceptance_queue.py", "# fixture\n")
    write_text(
        root / "scripts/storybook_manual_acceptance_smoke.py",
        'EVIDENCE_PATH = "target/storybook-manual-acceptance-evidence.json"\n'
        "def manual_acceptance_evidence_report(manifest, audit):\n"
        "    entry = {'command': 'open', 'smoke_command': 'smoke', 'minimum_observation_frames': 48}\n"
        "    return [{\n"
        '        "command": entry.get("command"),\n'
        '        "smoke_command": entry.get("smoke_command"),\n'
        '        "minimum_observation_frames": entry.get("minimum_observation_frames"),\n'
        "    }]\n"
        "def write_evidence_report(manifest, audit, evidence):\n"
        "    return manual_acceptance_evidence_report(manifest, audit)\n",
    )
    write_text(root / "scripts/test_storybook_manual_acceptance_smoke.py", "# fixture\n")
    write_text(
        root / "docs/storybook-77ui-deep-audit-ledger.md",
        "## UI: progress-bar\n"
        "target/manual-ui-probe/native-matrix-expanded-v3/summary.json\n"
        "progress_preview_click\n"
        "progress_timed_tick\n"
        "progress_timed_cycle\n"
        "progress_bar_timed_tick_advances_via_core_progress_action\n"
        "progress_bar_timed_tick_cycles_after_reaching_maximum\n"
        "progress_bar_live_audit_reports_timed_tick_progress_contract\n"
        "progress_bar_live_audit_reports_timed_cycle_after_maximum\n"
        "progress_bar_live_audit_reports_indeterminate_segment_motion\n"
        "progress_bar_indeterminate_segment_moves_on_runtime_tick\n"
        "progress_bar_window_runtime_tick_repaints_meter_body\n"
        "progress_bar_window_runtime_tick_cycles_after_maximum\n"
        "progress_bar_dedicated_render_uses_core_progress_bar_public_api\n",
    )
    write_text(
        root / "docs/storybook-77ui-repair-plan.md",
        "## UI: progress-bar\n"
        "rtk cargo run --release -p katana-ui-core-storybook --bin "
        "katana-ui-core-storybook --locked -- --open-window progress-bar\n"
        "rtk just storybook-manual-acceptance-smoke\n"
        "rtk cargo run --release -p katana-ui-core-storybook --bin "
        "katana-ui-core-storybook --locked -- --open-window 48 progress-bar\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/requirements.rs",
        'const REQUIRED_PAGES: &[&str] = &["text", "button"];\nconst MIN_SINGLE_NODE: usize = 1;\n',
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/catalog/preset_labels.rs",
        'fn for_page(page: &str) { match page { "text" => &["a","b","c","d"], "button" => &["a","b","c","d"], _ => &[] } }\n',
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/storybook_ui_option_contract.rs",
        option_contract_source(option_arm),
    )
    write_minimal_manifest(root, pages=("text", "button"))
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/inspector_rows.rs",
        "fn x() { storybook_ui_option_contract::settings_rows_for(); }\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/mod.rs",
        "mod visual_inspector_option_contract_tests;\n"
        "mod visual_inspector_button_preset_tests;\n"
        "mod visual_inspector_preset_follow_tests;\n"
        "mod visual_inspector_text_entry_preset_tests;\n"
        "mod visual_inspector_fallback_status_tests;\n"
        "mod visual_preset_tab_scroll_tests;\n"
        "mod visual_interaction_button_instance_tests;\n"
        "mod visual_interaction_binary_choice_state_tests;\n"
        "mod visual_interaction_text_entry_options_tests;\n"
        "mod visual_interaction_surface_options_tests;\n"
        "mod visual_interaction_foundation_options_tests;\n"
        "mod visual_interaction_foundation_extra_options_tests;\n"
        "mod visual_interaction_skeleton_options_tests;\n"
        "mod visual_interaction_split_pane_options_tests;\n"
        "mod visual_interaction_layout_options_tests;\n"
        "mod visual_interaction_primitive_options_tests;\n"
        "mod visual_interaction_binary_choice_options_tests;\n"
        "mod visual_interaction_banner_options_tests;\n"
        "mod visual_interaction_feedback_options_tests;\n"
        "mod visual_interaction_collection_options_tests;\n"
        "mod visual_interaction_navigation_options_tests;\n"
        "mod visual_interaction_overlay_options_tests;\n"
        "mod visual_interaction_toolbar_options_tests;\n"
        "mod visual_interaction_settings_list_options_tests;\n"
        "mod visual_interaction_color_picker_options_tests;\n"
        "mod visual_interaction_virtualization_options_tests;\n"
        "mod visual_interaction_search_control_options_tests;\n"
        "mod visual_interaction_status_bar_options_tests;\n"
        "mod visual_interaction_chip_options_tests;\n"
        "mod visual_interaction_chip_family_options_tests;\n"
        "mod visual_interaction_icon_options_tests;\n"
        "mod visual_interaction_closeable_tab_strip_options_tests;\n"
        "mod visual_interaction_tabs_options_tests;\n"
        "mod visual_interaction_selection_instance_tests;\n"
        "mod visual_interaction_selection_list_preset_tests;\n"
        "mod visual_interaction_menu_tests;\n"
        "mod visual_interaction_side_menu_tests;\n"
        "mod visual_interaction_tree_view_tests;\n"
        "mod visual_interaction_context_menu_tests;\n"
        "mod visual_interaction_selection_options_tests;\n"
        "mod visual_interaction_command_palette_options_tests;\n"
        "mod visual_interaction_shortcut_cheatsheet_options_tests;\n"
        "mod panel_in_panel_state_tests;\n"
        "mod visual_interaction_runtime_options_tests;\n"
        "mod visual_interaction_runtime_structured_options_tests;\n"
        "mod visual_interaction_live_component_options_tests;\n"
        "mod visual_interaction_dynamic_array_editor_state_tests;\n"
        "mod visual_interaction_diagnostics_list_options_tests;\n"
        "mod visual_interaction_drag_and_drop_state_tests;\n"
        "mod visual_interaction_text_input_state_tests;\n"
        "mod visual_interaction_text_area_state_tests;\n"
        "mod visual_interaction_breadcrumb_state_tests;\n"
        "mod visual_interaction_status_bar_state_tests;\n"
        "mod visual_interaction_toolbar_state_tests;\n"
        "mod visual_interaction_tabs_state_tests;\n"
        "mod visual_interaction_closeable_tab_strip_state_tests;\n"
        "mod visual_interaction_closeable_tab_strip_context_tests;\n"
        "mod visual_interaction_toggle_state_tests;\n"
        "mod visual_interaction_segmented_toggle_state_tests;\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/screen_state_setting_semantics.rs",
        "fn semantic_setting_state() {\n"
        "    \"toolbar\"; \"settings-list\"; \"color-picker-rgba\";\n"
        "    \"theme-tokens\";\n"
        "    \"text\"; \"skeleton\"; \"loading-dots\"; \"spinner\";\n"
        "    \"key-cap\"; \"motion\";\n"
        "    \"progress-bar\";\n"
        "    \"split-pane\";\n"
        "    \"scroll-area\"; \"align-center\";\n"
        "    \"divider\"; \"spacer\"; \"color-swatch\"; \"slide-control\";\n"
        "    \"checkbox\"; \"radio\"; \"toggle\"; \"segmented-toggle\";\n"
        "    \"icon\";\n"
        "    \"text-input\"; \"text-area\";\n"
        "    \"badge\"; \"banner\"; \"card\"; \"empty-state\";\n"
        "    \"toast-stack-manager\"; \"notification-toast\";\n"
        "    \"hover-card\"; \"menu\"; \"form-field\"; \"breadcrumb\";\n"
        "    \"side-menu\"; \"list\"; \"collapsible-panel\"; \"tree-view\";\n"
        "    \"panel\";\n"
        "    \"virtualization\"; \"search-control-strip\";\n"
        "    \"status-bar\";\n"
        "    \"chip\";\n"
        "    \"attachment-chip\"; \"chip-group\";\n"
        "    \"command-palette\";\n"
        "    \"shortcut-cheatsheet\";\n"
        "    \"context-menu\"; \"startup-state-panel\"; \"code-diff\";\n"
        "    \"shortcut-combo\"; \"skeleton-cluster\";\n"
        "    \"window-control-button-group\"; \"accordion\";\n"
        "    \"tooltip\"; \"popover\"; \"modal\"; \"modal-overlay\";\n"
        "    \"diagnostics-list\";\n"
        "    \"dynamic-array-editor\"; \"drag-and-drop\";\n"
        "    \"combo-box\"; \"select-box\"; \"selection-list\";\n"
        "    \"menu-button\"; \"search-box\";\n"
        "    toolbar.action.disabled=true;\n"
        "    settings_list.control.options=4;\n"
        "    settings_list.label=Workspace settings;\n"
        "    settings_list.dirty=Highlight;\n"
        "    settings_list.section.description=visible;\n"
        "    settings_list.field.label=Font size;\n"
        "    settings_list.control.kind=Number;\n"
        "    color_picker.eyedropper=storybook-eyedropper;\n"
        "    color_picker.rgba=rgba(64,128,255,.8);\n"
        "    color_picker.color_area=saturation/value;\n"
        "    color_picker.trigger.border=false;\n"
        "    text.script=jp+emoji;\n"
        "    skeleton.aspect_ratio=16:9;\n"
        "    progress_bar.percent=82;\n"
        "    loading_dots.dot_count=5;\n"
        "    spinner.animation_state=Paused;\n"
        "    theme.color.accent=green;\n"
        "    key_cap.theme.color=accent;\n"
        "    motion.reduced_policy=ForceReduced;\n"
        "    split_pane.resize_mode=KeyboardOnly;\n"
        "    scroll_area.overflow=scroll;\n"
        "    align_center.alignment=center;\n"
        "    divider.variant=alternate;\n"
        "    color_swatch.tone=accent;\n"
        "    slide_control.theme.slot=custom;\n"
        "    checkbox.checked=true;\n"
        "    radio.focus=visible;\n"
        "    toggle.disabled=true;\n"
        "    segmented_toggle.selected=true;\n"
        "    icon.svg_source=custom-svg;\n"
        "    icon.paint_policy=currentColor;\n"
        "    icon.theme_token=muted;\n"
        "    text_input.value=typed 日本語;\n"
        "    text_area.resize_enabled=true;\n"
        "    badge.leading_icon=dot;\n"
        "    banner.leading_icon=custom;\n"
        "    toast_stack.duration=custom;\n"
        "    notification_toast.action=visible;\n"
        "    list.virtualization=visible_range;\n"
        "    collapsible_panel.resize_handle=true;\n"
        "    hover_card.pointer_follow=true;\n"
        "    panel.horizontal_scroll=changed;\n"
        "    panel.nested_state=independent;\n"
        "    menu.selected_index=1;\n"
        "    menu.panel_placement=resolved;\n"
        "    form_field.helper_text=long;\n"
        "    breadcrumb.crumb_action=callback;\n"
        "    side_menu.hover_expansion=true;\n"
        "    tree.context_menu=enabled;\n"
        "    tooltip.open=true;\n"
        "    popover.placement=edge;\n"
        "    modal.focus=first;\n"
        "    modal_overlay.dismiss=outside;\n"
        "    card.child_state=changed;\n"
        "    empty_state.actions=Primary+Secondary;\n"
        "    virtualization.viewport.offset=1260;\n"
        "    virtualization.overscan=4;\n"
        "    search_control.regex=true;\n"
        "    search_control.query=heading;\n"
        "    search_control.result_count=0;\n"
        "    status_bar.segment_a11y=custom;\n"
        "    chip.a11y_label=Filter chip;\n"
        "    attachment.retry=visible;\n"
        "    chip_group.overflow_trigger_width=32;\n"
        "    command_palette.provider_group=workspace/editor/app;\n"
        "    shortcut_cheatsheet.query=カテゴリ;\n"
        "    context_menu.placement_used=AboveEnd;\n"
        "    startup_state.retry=true;\n"
        "    code_diff.scroll_sync=false;\n"
        "    shortcut_combo.platform_display=MacOS;\n"
        "    skeleton_cluster.reduced_motion=true;\n"
        "    window_control.visibility=Hover;\n"
        "    accordion.trigger_area=full-row;\n"
        "    diagnostics.bulk_action=Apply;\n"
        "    array.order=2,1,3;\n"
        "    array.theme_row=accent;\n"
        "    drag.drop_indicator=after;\n"
        "    drag.keyboard_draggable=true;\n"
        "    combo.outside_click_dismiss=true;\n"
        "    selection_list.more_row=true;\n"
        "    menu_button.select_action=callback;\n"
        "    search_box.regex_case=true/true;\n"
        "}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/screen_state.rs",
        "fn register() { semantic_setting_state(page, option); }\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/text_input_screen_state.rs",
        "struct TextInputStateStore { instances: BTreeMap<&'static str, TextInputRuntimeState> }\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/screen_state_text_input.rs",
        "fn register_text_input_readonly_block() {}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/text_area_screen_state.rs",
        "const DEFAULT_TEXT_AREA_INSTANCE: &str = \"text-area.preview\";\n"
        "struct TextAreaStateStore { instances: BTreeMap<&'static str, TextAreaRuntimeState> }\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/screen_state_text_area.rs",
        "fn text_area_runtime_mut_for() {}\n"
        "fn text_area_runtime_for() {}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/screen_state_text_area_scroll.rs",
        "fn text_area_runtime_mut_for() {}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/screen_state_text_area_queries.rs",
        "fn text_area_runtime_for() {}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/preset_tab_scroll.rs",
        "fn max_scroll_x_for_page() {}\n"
        "fn scroll_delta() { clamp_offset(); viewport_rect(); visible_index_range(); }\n"
        "fn ensure_index_visible() {\n"
        "    if tab_left < offset {}\n"
        "    if tab_right > offset + viewport_width() {}\n"
        "}\n"
        "fn active_index_scroll_x() {}\n"
        "fn hit_index_at() {\n"
        "    viewport.contains(x, y);\n"
        "    visual_rect_for_index(page, index, false, scroll_x);\n"
        "    rect.contains(x, y);\n"
        "}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/preset_tabs.rs",
        "fn draw() { canvas.with_clip(); preset_tab_label::fit(); }\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/preset_tab_label.rs",
        "fn fit() { measure_width(); }\n"
        "const TRUNCATION_MARKER: &str = \"...\";\n"
        "fn measured_width_for_test() {}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/visual_preset_tab_scroll_tests.rs",
        "fn overflowing_preset_tabs_have_horizontal_scroll_range() {}\n"
        "fn visible_preset_tab_rects_stay_fully_inside_viewport() {}\n"
        "fn rendered_preset_tabs_are_clipped_at_preview_right_edge() {\n"
        "    pixel_at(&canvas);\n"
        "}\n"
        "fn external_preset_selection_scrolls_current_tab_into_view() {\n"
        "    state.select_preset(last_preset);\n"
        "    active_tab_is_inside_viewport();\n"
        "}\n"
        "fn clicking_scrolled_preset_tab_uses_logical_tab_index() {}\n"
        "fn wheel_over_preset_tabs_scrolls_tabs_without_scrolling_root() {\n"
        "    apply_scroll_delta_at_for_test();\n"
        "    state.scroll_y;\n"
        "}\n"
        "fn external_render_preset_scrolls_active_overflow_tab_into_view() {}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/visual_inspector_option_contract_tests.rs",
            "fn inspector_settings_rows_include_every_option_contract_for_each_story() {"
            "fn inspector_setting_rows_apply_each_clicked_option_contract() {"
            "fn inspector_setting_rows_repaint_representative_preview_contracts() {"
            "fn button_option_controls_match_storybook_option_contract() {"
        "StorybookButtonOptionControl::all();"
        "control.setting_name();"
        "}"
            "fn button_inspector_controls_apply_each_button_option_contract() {"
        "button_options::control_rect();"
        "StorybookButtonOptionControl::all();"
        "control.setting_name();"
        "}"
        "fn text_entry_inspector_rows_select_matching_preset_tabs() {"
        "state.preset_index;"
        "text_area.vertical_scrollbar_visible;"
        "text_area.horizontal_scrollbar_visible;"
        "}"
        "StoryCatalog.examples();"
        "storybook_ui_option_contract::options_for_page();"
        "inspector_rows::settings_rows();"
        "apply_click(&mut state;"
        "option.after;"
        "setting_is_visible();"
        "ROW_MAX_CHARS;"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_inspector_button_preset_tests.rs",
        "fn button_inspector_rows_select_matching_preset_tabs() {"
        "button_options::preset_index_for_control();"
        "state.preset_index;"
        "}\n"
        "fn button_inspector_rows_apply_action_event_and_state_for_every_button_page() {"
        "button_option_apply;"
        "button_option_changed;"
        "control.state_label(state.screen_state.button_options);"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_inspector_preset_follow_tests.rs",
        "fn inspector_rows_select_preset_tabs_for_every_non_button_option() {"
        "expected_preset_index(page, option.setting, option_index);"
        "state.preset_index;"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_inspector_text_entry_preset_tests.rs",
        "fn inspector_rows_select_matching_preset_tabs_for_option_focused_pages() {"
        "state.preset_index;"
        "text_area.vertical_scrollbar_visible;"
        "text_area.horizontal_scrollbar_visible;"
        "text_area.leading_slot.icon;"
        "text_area.trailing_icon_buttons;"
        "text_area.clear_action;"
        "tabs.active_scroll;"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_inspector_fallback_status_tests.rs",
        "fn required_page_inspector_options_do_not_use_generic_fallback_status() {"
        "assert_not_generic_inspector_fallback();"
        "option.after;"
        "option.setting, state.screen_state.state_label;"
        "}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/dedicated.rs",
        'fn draw_page(page: &str) { match page { "text" => text(), "button" => button(), _ => draw() } }\n',
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/visual_text_tests.rs",
        'fn text_presets_render_distinct_bodies() { let page = "text"; }\n',
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/visual_button_tests.rs",
        'fn button_presets_render_distinct_bodies() { let page = "button"; }\n',
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/visual_interaction_button_center_tests.rs",
        "fn button_label_center_uses_measured_text_width() {"
        "measure_button_label_width();"
        "centered_label_x_for_test();"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_button_instance_tests.rs",
        "fn button_family_window_interaction_keeps_instance_state_isolated_across_presets() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    StorybookButtonOptionControl::Label;\n"
        "    button_option_apply;\n"
        "    button_pressed;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_text_entry_options_tests.rs",
        "fn text_input_inspector_options_mutate_value_slot_icon_and_blocking_semantic_state() {\n"
        "    assert_inspector_option_contract_state;\n"
        "    text_input.leading_slot.icon=search-svg;\n"
        "}\n"
        "fn text_area_inspector_options_mutate_multiline_scroll_slot_and_blocking_semantic_state() {\n"
        "    text_area.horizontal_scrollbar_visible=true;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_foundation_options_tests.rs",
        "fn text_inspector_options_mutate_role_script_metrics_and_wrap_semantic_state() {\n"
        "    assert_inspector_option_state;\n"
        "    text.script=jp+emoji;\n"
        "}\n"
        "fn progress_bar_inspector_options_mutate_progress_loading_tone_and_size_semantic_state() {\n"
        "    progress_bar.percent=82;\n"
        "}\n"
        "fn loading_indicator_inspector_options_mutate_animation_label_tone_and_size_semantic_state() {\n"
        "    loading_dots.dot_count=5;\n"
        "    spinner.animation_state=Paused;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_foundation_extra_options_tests.rs",
        "fn foundation_extra_inspector_options_mutate_theme_key_cap_and_motion_semantic_state() {\n"
        "    assert_inspector_option_state;\n"
        "    theme.color.accent=green;\n"
        "    key_cap.theme.color=accent;\n"
        "    motion.reduced_policy=ForceReduced;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_skeleton_options_tests.rs",
        "fn skeleton_inspector_options_mutate_shape_motion_size_and_a11y_semantic_state() {\n"
        "    assert_inspector_option_state;\n"
        "    skeleton.line_thickness=12;\n"
        "    skeleton.reduced_motion=true;\n"
        "    skeleton.aspect_ratio=16:9;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_split_pane_options_tests.rs",
        "fn split_pane_inspector_options_mutate_axis_ratio_bounds_and_resize_semantic_state() {\n"
        "    assert_inspector_option_state;\n"
        "    split_pane.ratio_percent=64;\n"
        "    split_pane.handle_width_px=10;\n"
        "    split_pane.resize_mode=KeyboardOnly;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_layout_options_tests.rs",
        "fn layout_inspector_options_mutate_axis_gap_alignment_and_overflow_semantic_state() {\n"
        "    assert_layout_option_state;\n"
        "    assert_inspector_option_state_with_event;\n"
        "    layout_option_changed;\n"
        "    row.alignment=center;\n"
        "    grid.overflow=scroll;\n"
        "    scroll_area.overflow=scroll;\n"
        "    align_center.alignment=center;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_primitive_options_tests.rs",
        "fn primitive_inspector_options_mutate_variant_tone_size_and_theme_slot_semantic_state() {\n"
        "    assert_inspector_option_state;\n"
        "    divider.variant=alternate;\n"
        "    color_swatch.tone=accent;\n"
        "    slide_control.theme.slot=custom;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_binary_choice_options_tests.rs",
        "fn binary_choice_inspector_options_mutate_selected_disabled_focus_and_checked_semantic_state() {\n"
        "    checkbox.checked=true;\n"
        "    radio.focus=visible;\n"
        "    toggle.disabled=true;\n"
        "    segmented_toggle.selected=true;\n"
        "    assert_component_state(page, setting, &state.screen_state);\n"
        "    checkbox_state_snapshot;\n"
        "    radio_state_snapshot;\n"
        "    assert_binary_component_state;\n"
        "    state.common.disabled;\n"
        "    state.interaction.focused;\n"
        "    settings_disabled;\n"
        "    selection_settings_changed;\n"
        "}\n"
        "fn binary_choice_disabled_option_blocks_preview_mutation() {\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_icon_options_tests.rs",
        "fn icon_inspector_options_mutate_svg_source_role_paint_and_token_semantic_state() {\n"
        "    assert_inspector_option_contract_state;\n"
        "    icon.svg_source=custom-svg;\n"
        "    icon.paint_policy=currentColor;\n"
        "    icon.theme_token=muted;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_closeable_tab_strip_options_tests.rs",
        "fn closeable_tab_strip_inspector_options_mutate_active_overflow_pin_and_group_semantic_state() {\n"
        "    tabs.active=settings;\n"
        "    tabs.pinned=true left-fixed;\n"
        "    tabs.group=Docs;\n"
        "    tabs.overflow=menu;\n"
        "    assert_closeable_tab_event;\n"
        "    state.screen_state.last_action;\n"
        "    state.screen_state.last_event;\n"
        "    starts_with(\"closeable_tab\");\n"
        "}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/visual_interaction_tabs_options_tests.rs",
        "fn tabs_inspector_options_mutate_tab_model_state() {\n"
        "    tabs.count=6 active=notes.md;\n"
        "    tabs.pinned=true left-fixed;\n"
        "    tabs.group=Docs;\n"
        "    tabs.overflow=menu;\n"
        "    tabs.active_scroll=follow;\n"
        "    assert_tabs_option_event;\n"
        "    state.screen_state.last_action;\n"
        "    state.screen_state.last_event;\n"
        "    starts_with(\"closeable_tab\");\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_surface_options_tests.rs",
        "fn badge_inspector_options_mutate_status_size_icon_and_variant_semantic_state() {\n"
        "    assert_inspector_option_contract_state;\n"
        "    badge.leading_icon=dot;\n"
        "}\n"
        "fn card_inspector_options_mutate_slot_click_and_child_semantic_state() {\n"
        "    card.child_state=changed;\n"
        "}\n"
        "fn empty_state_inspector_options_mutate_content_alignment_and_action_semantic_state() {\n"
        "    empty_state.actions=Primary+Secondary;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_banner_options_tests.rs",
        "fn banner_inspector_options_mutate_feedback_details_icon_and_placement_semantic_state() {\n"
        "    assert_inspector_option_state;\n"
        "    banner.severity=warning;\n"
        "    banner.details=expanded;\n"
        "    banner.leading_icon=custom;\n"
        "    banner.placement=sticky;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_feedback_options_tests.rs",
        "fn feedback_inspector_options_mutate_severity_duration_action_and_dismiss_semantic_state() {\n"
        "    assert_inspector_option_state;\n"
        "    toast_stack.duration=custom;\n"
        "    notification_toast.action=visible;\n"
        "    notification_toast.dismiss=true;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_collection_options_tests.rs",
        "fn collection_inspector_options_mutate_list_collapsible_hover_and_panel_semantic_state() {\n"
        "    assert_collection_option_state;\n"
        "    assert_inspector_option_state_with_event;\n"
        "    panel_active_select;\n"
        "    panel_scrollbar_hide;\n"
        "    list.virtualization=visible_range;\n"
        "    collapsible_panel.resize_handle=true;\n"
        "    hover_card.pointer_follow=true;\n"
        "    panel.horizontal_scroll=changed;\n"
        "    panel.nested_state=independent;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_navigation_options_tests.rs",
        "fn navigation_inspector_options_mutate_menu_form_breadcrumb_side_and_tree_semantic_state() {\n"
        "    assert_navigation_option_state;\n"
        "    breadcrumb_click;\n"
        "    field_validate;\n"
        "    form_field_helper_text;\n"
        "    menu.selected_index=1;\n"
        "    menu.panel_placement=resolved;\n"
        "    form_field.helper_text=long;\n"
        "    breadcrumb.crumb_action=callback;\n"
        "    side_menu.hover_expansion=true;\n"
        "    tree.context_menu=enabled;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_overlay_options_tests.rs",
        "fn tooltip_inspector_options_mutate_overlay_semantic_state() {\n"
        "    assert_inspector_option_state;\n"
        "    tooltip.open=true;\n"
        "}\n"
        "fn popover_inspector_options_mutate_overlay_semantic_state() {\n"
        "    popover.placement=edge;\n"
        "}\n"
        "fn modal_inspector_options_mutate_overlay_semantic_state() {\n"
        "    modal.focus=first;\n"
        "}\n"
        "fn modal_overlay_inspector_options_mutate_overlay_semantic_state() {\n"
        "    modal_overlay.dismiss=outside;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_toolbar_options_tests.rs",
        "fn toolbar_inspector_options_mutate_action_split_and_group_semantic_state() {\n"
        "    assert_inspector_option_contract_state;\n"
        "    toolbar.action.disabled=true;\n"
        "    toolbar.split.a11y=Open menu;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_settings_list_options_tests.rs",
        "fn settings_list_inspector_options_mutate_field_control_and_reset_semantic_state() {\n"
        "    settings_list.control.options=4;\n"
        "    settings_list.label=Workspace settings;\n"
        "    settings_list.dirty=Highlight;\n"
        "    settings_list.section.description=visible;\n"
        "    settings_list.field.label=Font size;\n"
        "    settings_list.control.kind=Number;\n"
        "    settings_list.reset=default;\n"
        "    assert_settings_list_runtime;\n"
        "    option_state();\n"
        "    options.label_workspace;\n"
        "    options.density_compact;\n"
        "    options.dirty_highlight;\n"
        "    options.sections_app_lint;\n"
        "    options.section_label_editor;\n"
        "    options.section_description_visible;\n"
        "    options.section_icon_gear;\n"
        "    options.field_count;\n"
        "    options.section_footer_policy;\n"
        "    options.section_collapsible;\n"
        "    options.default_collapsed;\n"
        "    options.field_label_font_size;\n"
        "    options.field_description_visible;\n"
        "    options.control_kind_number;\n"
        "    options.control_option_count;\n"
        "    options.custom_control_button;\n"
        "    options.value_changed;\n"
        "    options.reset_default;\n"
        "}\n"
        "fn settings_list_window_interaction_keeps_query_field_collapse_and_reset_instance_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    settings_filter_update_collapse;\n"
        "    settings_field_changed;\n"
        "    settings_update_field;\n"
        "    settings_reset_field;\n"
        "    has_dirty_font_size;\n"
        "    has_query_filter;\n"
        "    has_collapsed_chat_section;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_color_picker_options_tests.rs",
        "fn color_picker_inspector_options_mutate_hue_alpha_block_and_callback_semantic_state() {\n"
        "    color_picker.rgba=rgba(64,128,255,.8);\n"
        "    color_picker.color_area=saturation/value;\n"
        "    color_picker.trigger.border=false;\n"
        "    color_picker.readonly.blocks_writes;\n"
        "    color_picker.disabled.blocks_focus;\n"
        "    assert_color_picker_runtime;\n"
        "    option_state();\n"
        "    options.panel_open;\n"
        "    options.blending_multiply;\n"
        "    options.color_area_visible;\n"
        "    options.trigger_large;\n"
        "    options.title_customized;\n"
        "    options.panel_scale_percent;\n"
        "    options.trigger_border;\n"
        "}\n"
        "fn color_picker_window_interaction_keeps_drag_value_callback_and_blocked_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    color_drag;\n"
        "    rgba_changed;\n"
        "    color_picker.rgba_label();\n"
        "    has_committed_color;\n"
        "    callback_action();\n"
        "    component_body_pixel_diff();\n"
        "}\n"
        "fn color_picker_readonly_and_disabled_preview_clicks_do_not_mutate_color() {\n"
        "    color_picker_readonly_blocked;\n"
        "    color_picker_disabled_blocked;\n"
        "    blocks_writes;\n"
        "    blocks_focus;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_virtualization_options_tests.rs",
        "fn virtualization_inspector_options_mutate_range_focus_and_measurement_semantic_state() {\n"
        "    assert_inspector_option_contract_state;\n"
        "    virtualization.focused_index=42;\n"
        "    virtualization.measured_correction=+8;\n"
        "    virtualization.overscan=4;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_search_control_options_tests.rs",
        "fn search_control_inspector_options_mutate_match_replace_and_active_result_semantic_state() {\n"
        "    assert_inspector_option_contract_state;\n"
        "    search_control.query=heading;\n"
        "    search_control.match_case=true;\n"
        "    search_control.result_count=0;\n"
        "    search_control.active_index=none;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_status_bar_options_tests.rs",
        "fn status_bar_inspector_options_mutate_segment_and_message_semantic_state() {\n"
        "    assert_inspector_option_contract_state;\n"
        "    status_bar.progress_popover=true;\n"
        "    status_bar.segment_a11y=custom;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_chip_options_tests.rs",
        "fn chip_inspector_options_mutate_label_icon_variant_and_state_semantic_state() {\n"
        "    assert_inspector_option_contract_state;\n"
        "    chip.leading_icon=tag;\n"
        "    chip.a11y_label=Filter chip;\n"
        "    chip.focused=true;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_chip_family_options_tests.rs",
        "fn attachment_chip_inspector_options_mutate_kind_status_and_retry_semantic_state() {\n"
        "    assert_inspector_option_contract_state;\n"
        "    attachment.retry=visible;\n"
        "}\n"
        "fn chip_group_inspector_options_mutate_overflow_reorder_and_width_semantic_state() {\n"
        "    chip_group.overflow_trigger_width=32;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_command_palette_options_tests.rs",
        "fn command_palette_inspector_options_mutate_query_highlight_provider_semantic_state() {\n"
        "    settings_command_palette_option;\n"
        "    molecule_settings_changed;\n"
        "    state.screen_state.last_setting_value;\n"
        "    command_palette.query=theme;\n"
        "    command_palette.highlight=2;\n"
        "    command_palette.provider_group=workspace/editor/app;\n"
        "    assert_command_palette_runtime;\n"
        "    option_state();\n"
        "    command_palette.query();\n"
        "    command_palette.highlighted_index();\n"
        "    options.row_count;\n"
        "    options.provider_group_workspace_editor_app;\n"
        "    options.shortcut_display_visible;\n"
        "}\n"
        "fn command_palette_window_interaction_keeps_query_and_highlight_instance_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    command_palette.query=theme;\n"
        "    command_palette.highlight=2;\n"
        "    assert_command_palette_runtime;\n"
        "    option_state();\n"
        "    settings_command_palette_option;\n"
        "    molecule_settings_changed;\n"
        "    state.screen_state.last_setting_value;\n"
        "    command_palette.query();\n"
        "    command_palette.highlighted_index();\n"
        "    options.row_count;\n"
        "    options.provider_group_workspace_editor_app;\n"
        "    options.shortcut_display_visible;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/panel_in_panel_state_tests.rs",
        "fn panel_window_interaction_keeps_instance_scroll_and_nested_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    PanelChildKey::Details;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_shortcut_cheatsheet_options_tests.rs",
        "fn shortcut_cheatsheet_inspector_options_mutate_filter_selection_and_count_semantic_state() {\n"
        "    settings_shortcut_cheatsheet_option;\n"
        "    runtime_settings_changed;\n"
        "    state.screen_state.last_setting_value;\n"
        "    shortcut_cheatsheet.query=カテゴリ;\n"
        "    shortcut_cheatsheet.selected=format;\n"
        "    shortcut_cheatsheet.result_count=1;\n"
        "    assert_shortcut_cheatsheet_runtime;\n"
        "    option_state();\n"
        "    options.label_editor_keys;\n"
        "    options.group_count;\n"
        "    options.item_count;\n"
        "    options.group_layout_one_column;\n"
        "    options.query_category;\n"
        "    options.selected_format;\n"
        "    options.result_count;\n"
        "    cheatsheet.visible_item_count();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_runtime_options_tests.rs",
        "fn context_menu_inspector_options_mutate_anchor_placement_and_size_semantic_state() {\n"
        "    assert_inspector_option_contract_state;\n"
        "    context_menu.placement_used=AboveEnd;\n"
        "}\n"
        "fn startup_state_inspector_options_mutate_error_progress_and_action_semantic_state() {\n"
        "    startup_state.retry=true;\n"
        "}\n"
        "fn code_diff_inspector_options_mutate_mode_layout_and_sync_semantic_state() {\n"
        "    code_diff.scroll_sync=false;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_runtime_structured_options_tests.rs",
        "fn shortcut_combo_inspector_options_mutate_display_size_tone_and_a11y_semantic_state() {\n"
        "    expected_action(page);\n"
        "    runtime_settings_changed;\n"
        "    assert_runtime_structured_state;\n"
        "    runtime_structured.shortcut_combo;\n"
        "    platform_display_macos;\n"
        "    shortcut_combo.platform_display=MacOS;\n"
        "}\n"
        "fn skeleton_cluster_inspector_options_mutate_preset_children_and_motion_semantic_state() {\n"
        "    runtime_structured.skeleton_cluster;\n"
        "    reduced_motion;\n"
        "    skeleton_cluster.reduced_motion=true;\n"
        "}\n"
        "fn window_control_inspector_options_mutate_position_size_controls_and_visibility_semantic_state() {\n"
        "    runtime_structured.window_control;\n"
        "    visibility_hover;\n"
        "    window_control.visibility=Hover;\n"
        "}\n"
        "fn accordion_inspector_options_mutate_controlled_trigger_and_motion_semantic_state() {\n"
        "    runtime_structured.accordion;\n"
        "    trigger_area_full_row;\n"
        "    accordion.trigger_area=full-row;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_live_component_options_tests.rs",
        "fn live_component_inspector_options_mutate_array_and_drag_semantic_state() {\n"
        "    assert_live_component_runtime;\n"
        "    state.screen_state.last_event;\n"
        "    dynamic_array.item_count();\n"
        "    dynamic_array.order_label();\n"
        "    drag_and_drop.is_dragging();\n"
        "    drag_and_drop.committed();\n"
        "    array.order=2,1,3;\n"
        "    array.theme_row=accent;\n"
        "    drag.drop_indicator=after;\n"
        "    drag.keyboard_draggable=true;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_diagnostics_list_options_tests.rs",
        "fn diagnostics_list_inspector_options_mutate_filter_bulk_and_fix_preview_semantic_state() {\n"
        "    settings_diagnostics_option;\n"
        "    molecule_settings_changed;\n"
        "    state.screen_state.last_setting_value;\n"
        "    diagnostics.virtualization=Windowed;\n"
        "    diagnostics.bulk_action=Apply;\n"
        "    diagnostics.fix_preview=Collapsed;\n"
        "    assert_diagnostics_list_runtime;\n"
        "    option_state();\n"
        "    options.group_by_source;\n"
        "    options.sort_by_location;\n"
        "    options.severity_filter_error_only;\n"
        "    options.wrap_error_navigation_disabled;\n"
        "    options.virtualization_windowed;\n"
        "    options.bulk_action_apply;\n"
        "    options.fix_preview_collapsed;\n"
        "}\n"
        "fn diagnostics_list_window_interaction_keeps_filter_bulk_fix_preview_instance_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    diagnostic_fix_preview;\n"
        "    diagnostic_fix_preview_toggled;\n"
        "    diagnostic_bulk_apply;\n"
        "    has_error_filter;\n"
        "    has_bulk_applied;\n"
        "    has_fix_preview;\n"
        "    assert_diagnostics_list_runtime;\n"
        "    option_state();\n"
        "    settings_diagnostics_option;\n"
        "    molecule_settings_changed;\n"
        "    state.screen_state.last_setting_value;\n"
        "    options.group_by_source;\n"
        "    options.sort_by_location;\n"
        "    options.virtualization_windowed;\n"
        "    options.fix_preview_collapsed;\n"
        "    component_body_pixel_diff;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_selection_options_tests.rs",
        "fn combo_box_inspector_options_mutate_choice_semantic_state() {\n"
        "    assert_inspector_option_contract_state;\n"
        "    combo.outside_click_dismiss=true;\n"
        "}\n"
        "fn select_box_inspector_options_mutate_choice_semantic_state() {}\n"
        "fn selection_list_inspector_options_mutate_list_semantic_state() {\n"
        "    selection_list.more_row=true;\n"
        "}\n"
        "fn menu_button_inspector_options_mutate_menu_semantic_state() {\n"
        "    menu_button.select_action=callback;\n"
        "}\n"
        "fn search_box_inspector_options_mutate_search_semantic_state() {\n"
        "    search_box.regex_case=true/true;\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_binary_choice_state_tests.rs",
        "fn checkbox_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(CHECKBOX_PRIMARY_INSTANCE);\n"
        "    state.select_instance(CHECKBOX_SECONDARY_INSTANCE);\n"
        "    checkbox_toggle;\n"
        "    component_body_pixel_diff();\n"
        "}\n"
        "fn checkbox_window_interaction_disabled_toggle_does_not_mutate_state() {\n"
        "    DISABLED_PRESET_INDEX;\n"
        "    component_body_pixel_diff();\n"
        "}\n"
        "fn radio_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(RADIO_PRIMARY_INSTANCE);\n"
        "    state.select_instance(RADIO_SECONDARY_INSTANCE);\n"
        "    radio_select;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_dynamic_array_editor_state_tests.rs",
        "fn dynamic_array_editor_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    array_add;\n"
        "    array_reorder;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_drag_and_drop_state_tests.rs",
        "fn drag_and_drop_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    dragging=true;\n"
        "    committed=true;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_text_input_state_tests.rs",
        "fn text_input_preset_tab_switching_keeps_runtime_state_isolated() {}\n"
        "fn text_input_keyboard_routes_to_selected_instance_state() {\n"
        "    state.select_instance(\"text-input.primary\");\n"
        "    state.select_instance(\"text-input.secondary\");\n"
        "    apply_text_input_key();\n"
        "    text_input_value_for();\n"
        "    assert_ne!();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_text_area_state_tests.rs",
        "fn text_area_state_store_keeps_instance_value_focus_and_caret_isolated() {}\n"
        "fn text_area_keyboard_routes_to_selected_instance_state() {\n"
        "    state.select_instance(\"text-area.primary\");\n"
        "    state.select_instance(\"text-area.secondary\");\n"
        "    apply_text_area_key();\n"
        "    text_area_value_for();\n"
        "    assert_ne!();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_toggle_state_tests.rs",
        "fn toggle_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    toggle_change;\n"
        "    checked=true;\n"
        "    component_body_pixel_diff();\n"
        "}\n"
        "fn toggle_window_interaction_disabled_click_does_not_mutate_state() {\n"
        "    DISABLED_PRESET_INDEX;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_segmented_toggle_state_tests.rs",
        "fn segmented_toggle_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    segment_select;\n"
        "    segment=1;\n"
        "    component_body_pixel_diff();\n"
        "}\n"
        "fn segmented_toggle_window_interaction_disabled_click_does_not_mutate_state() {\n"
        "    DISABLED_PRESET_INDEX;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_button_hover_tests.rs",
        "fn hover_draws_visible_border_for_all_button_surfaces() {"
        "hover_border;"
        '"must not use text color";'
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_selection_instance_tests.rs",
        "fn select_box_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    component_body_pixel_diff();\n"
        "}\n"
        "fn combo_box_window_interaction_keeps_instance_state_isolated() {}\n"
        "fn search_box_window_interaction_keeps_instance_state_isolated() {}\n"
        "fn selection_list_window_interaction_keeps_instance_state_isolated() {}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_selection_list_preset_tests.rs",
        "fn selection_list_window_interaction_keeps_instance_state_isolated_across_presets() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    state.select_preset(MULTI_PRESET);\n"
        "    selection_list_multi_mask;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_side_menu_tests.rs",
        "fn side_menu_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    side_menu_select;\n"
        "    route=1 focus=1;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_tree_view_tests.rs",
        "fn tree_view_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    tree_click_toggle;\n"
        "    component_body_pixel_diff();\n"
        "}\n"
        "fn tree_view_context_menu_keeps_instance_state_isolated() {\n"
        "    tree_context_menu;\n"
        "    apply_context_click_for_test();\n"
        "}\n"
        "fn tree_view_setting_action_keeps_instance_setting_isolated() {\n"
        "    tree.context_menu=enabled;\n"
        "    layout_metrics::inspector_setting_row_hit_rect();\n"
        "}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/visual_interaction_menu_tests.rs",
        "fn menu_window_interaction_keeps_open_select_shortcut_instance_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    menu_open;\n"
        "    menu_select;\n"
        "    menu_shortcut_activate;\n"
        "    component_body_pixel_diff();\n"
        "}\n"
        "fn menu_shortcut_activation_keeps_instance_state_isolated() {}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_breadcrumb_state_tests.rs",
        "fn breadcrumb_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    breadcrumb_selected_index;\n"
        "    route=2;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_status_bar_state_tests.rs",
        "fn status_bar_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    status_bar_segment_popover;\n"
        "    open_popover=progress;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_toolbar_state_tests.rs",
        "fn toolbar_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    tool_toggle;\n"
        "    hovered_toolbar_action_index;\n"
        "    component_body_pixel_diff();\n"
        "}\n"
        "fn toolbar_window_interaction_disabled_action_does_not_mutate_state() {\n"
        "    ACTION_DISABLED_PRESET_INDEX;\n"
        "    component_body_pixel_diff();\n"
        "}\n"
        "fn toolbar_window_interaction_disabled_split_does_not_mutate_state() {\n"
        "    SPLIT_DISABLED_PRESET_INDEX;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_tabs_state_tests.rs",
        "fn tabs_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    TabsScreenAction::AddTab;\n"
        "    TabsScreenAction::TogglePinActive;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_closeable_tab_strip_state_tests.rs",
        "fn closeable_tab_strip_window_interaction_keeps_instance_state_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    TabsScreenAction::AddTab;\n"
        "    TabsScreenAction::CloseActive;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_context_menu_tests.rs",
        "fn context_menu_preview_submenu_and_item_selection_use_real_core_actions() {\n"
        "    apply_context_click_for_test();\n"
        "    dedicated_context_menu_popup::insert_row_rect();\n"
        "    dedicated_context_menu_popup::submenu_link_rect();\n"
        "    context_menu_submenu_opened;\n"
        "    context_menu_item_selected;\n"
        "}\n"
        "fn context_menu_window_interaction_keeps_context_action_instance_isolated() {\n"
        "    state.select_instance(PRIMARY_INSTANCE);\n"
        "    state.select_instance(SECONDARY_INSTANCE);\n"
        "    apply_context_click_for_test();\n"
        "    dedicated_context_menu_popup::insert_row_rect();\n"
        "    dedicated_context_menu_popup::submenu_link_rect();\n"
        "    context_menu_select_item;\n"
        "    component_body_pixel_diff();\n"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/window_interaction/context_click.rs",
        "fn context_menu_command_at() {}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/screen_state_context_menu.rs",
        "fn register_context_menu_submenu() { ContextMenuAction::OpenSubmenu; }\n"
        "fn register_context_menu_select_link() { ContextMenuAction::Activate; }\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/visual_interaction_closeable_tab_strip_context_tests.rs",
        "fn closeable_tab_strip_tab_context_menu_applies_workspace_tab_commands() {\n"
        "    CLOSE_OTHERS_INDEX;\n"
        "    CLOSE_ALL_INDEX;\n"
        "    CLOSE_RIGHT_INDEX;\n"
        "    CLOSE_LEFT_INDEX;\n"
        "    NEW_GROUP_INDEX;\n"
        "    MOVE_TO_GROUP_INDEX;\n"
        "    apply_context_click_for_test();\n"
        "    click_tab_context_command();\n"
        "}\n"
        "fn closeable_tab_strip_context_menu_keeps_pinned_tabs_fixed_until_unpinned() {}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/visual_interaction_menu_button_tests.rs",
        "fn menu_button_hover_draws_shared_button_family_border_token() {"
        "hover_border;"
        "ThemeSnapshot::dark;"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/window_interaction/button_operation.rs",
        'fn uses_clickable_preview_cursor(page: &str) { page == "menu-button"; }\n',
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/window_interaction/tests/button_operation_tests.rs",
        'const BUTTON_FAMILY_CURSOR_PAGES: &[&str] = &["button", "menu-button"];\n'
        "fn button_variant_hover_uses_pointer_cursor() {"
        "StorybookCursorStyle::PointingHand;"
        "}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/window_interaction/state_store.rs",
        "struct StorybookScreenStateKey { component_id: &'static str, preset_index: usize, instance_id: &'static str }\n"
        "fn save_instance() {}\n"
        "fn restore_instance() {}\n"
        "fn screen_state_store_keeps_page_and_preset_state_separate() {}\n"
        "fn screen_state_store_keeps_non_input_component_instances_separate() {}\n"
        "fn screen_state_store_keeps_selection_component_instances_separate() {}\n"
        "fn x() { TabsScreenAction::AddTab; SelectionScreenAction::ComboFilter; }\n"
        "fn screen_state_store_removes_default_state_for_page_preset_key_only() {}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/window_interaction/state_store_tests.rs",
        "fn every_required_page_keeps_screen_state_instances_separate() {\n"
        "    StoryRequirements::required_pages();\n"
        "    store.save_instance(page, 0, \"primary\", primary.clone());\n"
        "    store.restore_instance(page, 0, \"secondary\");\n"
        "}\n"
        "fn screen_state_store_removes_default_instance_key_only_for_required_pages() {}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/window_interaction/tests/required_page_tests.rs",
        "fn every_required_page_has_screen_action_and_settings_paths() {"
        "for page in StoryRequirements::required_pages() {"
        "StorybookInteractionSpec::for_page(page);"
        "preview_detail::component_action_hit_rect(page);"
        "click_rect();"
        "}}"
        "fn every_required_page_click_repaints_component_body() {"
        "component_body_pixel_diff();"
        "}"
        "fn every_required_page_setting_repaints_component_body() {}"
        "fn every_required_page_preset_tab_repaints_component_body() {}"
        "fn every_required_page_keeps_action_state_separate_from_other_pages() {}"
        "fn every_required_page_keeps_window_interaction_instances_separate() {}"
        "fn every_required_page_keeps_settings_state_separate_from_other_pages() {}"
        "fn every_required_page_keeps_action_and_settings_state_separate_between_presets() {"
        "other_preset_for(page, original_preset_index, stored_preset_index);"
        "}\n",
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/window_interaction/tests/preview_action_tests.rs",
        "fn side_menu_window_interaction_selects_route_and_repaints() {\n"
        "    \"side-menu\";\n"
        "    side_menu_select;\n"
        "    select_box_selected;\n"
        "    route=1 focus=1;\n"
        "    pixel_diff(&before, &after);\n"
        "}\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/catalog/story_paths_atoms.rs",
        'const PATHS: &[StoryPath] = &[StoryPath { page: "text" }, StoryPath { page: "button" }];\n',
    )
    write_text(
        root / "openspec/changes/establish-kuc-atoms-molecules-catalog/storybook-menu-change-split.md",
        "- `draw_page` page 別描画あり: 2\n"
        "- `draw_page` page 別描画未作成: 0\n"
        "\n"
        "| group | menu page | leaf change | input | status |\n"
        "| --- | --- | --- | --- | --- |\n"
        "| Atoms | `text` | `storybook-page-text` | test | page別描画あり |\n"
        "| Atoms | `button` | `storybook-page-button` | test | page別描画あり |\n",
    )
    write_text(
        root / "openspec/changes/establish-kuc-atoms-molecules-catalog/storybook-menu-priority-order.md",
        "| priority | menu page | leaf change | reason |\n"
        "| --- | --- | --- | --- |\n"
        "| SB-001 | `text` | `storybook-page-text` | test |\n"
        "| SB-002 | `button` | `storybook-page-button` | test |\n",
    )
    write_leaf_change(root, "storybook-page-text")
    write_leaf_change(root, "storybook-page-button")


def write_runtime_page_fixture(root: Path, complete: bool = False) -> None:
    write_text(
        root / "crates/katana-ui-core-storybook/src/requirements.rs",
        'const CANVAS_REQUIRED_PAGES: &[&str] = &["text", "button"];\n'
        'const INTERACTIVE_RUNTIME_PAGES: &[&str] = &["command-chrome"];\n',
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/lib.rs",
        "StorybookRoutes::default_routes(); interactive_runtime_pages();\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/window.rs",
        "command_chrome_runtime::handles_page(selected_page);\n"
        + ("command_chrome_runtime::open_window(frames);\n" if complete else ""),
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/command_chrome_runtime.rs",
        "CommandChromeStorybookApp; eframe::run_native;\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/command_chrome_app.rs",
        "CommandChromeStorybookApp; CommandChromeSurface;\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/command_chrome_surface.rs",
        "show_command_chrome; "
        + ("EguiCommandChromeAdapter;\n" if complete else ""),
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/command_chrome_script.rs",
        "run_scripted_sequence;\n",
    )


def write_text_command_root_runtime_fixture(
    root: Path, missing_token: str | None = None
) -> None:
    write_text(
        root / "crates/katana-ui-core-storybook/src/requirements.rs",
        'const CANVAS_REQUIRED_PAGES: &[&str] = &["text", "button"];\n'
        'const INTERACTIVE_RUNTIME_PAGES: &[&str] = &["text-command-root"];\n',
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/lib.rs",
        "StorybookRoutes::default_routes(); interactive_runtime_pages();\n",
    )
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/window.rs",
        "text_command_root_storybook::handles_page(selected_page);\n"
        "text_command_root_storybook::open_window(frames);\n",
    )
    source = """
struct TextCommandRootStorybookApp;
eframe::run_native;
EguiTextCommandSurfaceHostRoot;
let token = EguiTextCommandSurfaceHostProjectionEncoder::token(...);
EguiTextCommandSurfaceRootFactory::default().retain(token);
root.show(ui);
.forward_events_once(&mut forwarder);
consumed_once: receipt.consumed_once(),
forwarder_calls: forwarder.calls,
if sequence.steps.len() < 9 {
write_mp4;
decode_mp4;
"framemd5";
decoded_frame_count != sequence.steps.len();
let manifest_path = output_dir.join("text-command-root-manifest.json");
FullRootManifest::from_sequence;
event_receipt: EventReceiptEvidence;
frame_sequence_sha256;
decoder: DecoderEvidence;
encoder_capability_verified;
muxer_capability_verified;
""".strip()
    core_contract = (
        "opaque_tokens_and_transport_have_no_clone_or_serialize_derives\n"
        "compatibility_types_are_hidden\n"
    )
    storybook_contract = "full_root_storybook_uses_only_the_public_facade_root\n"
    if missing_token is not None:
        source = source.replace(missing_token, "", 1)
        core_contract = core_contract.replace(missing_token, "", 1)
        storybook_contract = storybook_contract.replace(missing_token, "", 1)
    write_text(
        root / "crates/katana-ui-core-storybook/src/visual/text_command_root_storybook.rs",
        source,
    )
    write_text(
        root / "crates/katana-ui-core/src/egui/text_command_surface/host_root/factory_api.rs",
        "pub fn retain(\n",
    )
    write_text(
        root / "crates/katana-ui-core/src/egui/text_command_surface/host_root/token_api.rs",
        "pub fn token(\n",
    )
    write_text(
        root / "crates/katana-ui-core/src/egui/text_command_surface/host_root/types.rs",
        "pub struct EguiTextCommandSurfacePresentationToken;\n"
        "pub struct EguiTextCommandSurfaceHostProjectionEncoder;\n",
    )
    write_text(
        root / "crates/katana-ui-core/tests/egui_host_root_facade_contract.rs",
        core_contract,
    )
    write_text(
        root
        / "crates/katana-ui-core-storybook/src/visual/text_command_root_storybook_tests.rs",
        storybook_contract,
    )
    write_text(
        root / "crates/katana-ui-core/tests/egui_text_command_root_contract.rs",
        "root_event_batch_forwards_once_and_returns_a_closed_receipt\n"
        "assert!(receipt.consumed_once())\n",
    )


def write_minimal_manifest(root: Path, pages: tuple[str, ...]) -> None:
    entries = ",\n".join(
        "{"
        f'"page":"{page}",'
        '"group":"test",'
        '"engine":"clickable",'
        '"public_props_options":["storybook_ui_option_contract::options_for_page"],'
        '"state":["screen_state"],'
        '"action":["preview_action"],'
        '"event":["interaction_event"],'
        '"callback":["core_callback"],'
        '"required_operations":["pointer","focus","hover"],'
        '"tests":{'
        '"window_interaction":["shared:window_interaction::required_page_tests"],'
        '"visual_interaction":["shared:visual_interaction_tests"],'
        '"guard":["shared:guard"]'
        "},"
        '"audit_status":"partial",'
        '"evidence":['
        + minimal_manifest_evidence(page)
        + "],"
        '"gaps":["not a full fixture"]'
        "}"
        for page in pages
    )
    write_text(
        root / "docs/storybook-77ui-interaction-manifest.json",
        "{"
        '"schema_version":1,'
        '"policy":{'
        '"harness_scope":"core_public_api_harness",'
        '"uses_storybook_only_state":false,'
        '"uses_inspector_only_change":false,'
        '"uses_preset_label_only_change":false,'
        '"bypasses_core_public_api":false'
        "},"
        f'"ui":[{entries}]'
        "}\n",
    )


def minimal_manifest_evidence(page: str) -> str:
    if page == "text":
        return (
            '"minimal test fixture",'
            '"interactive_page_text_does_not_start_display_text_selection",'
            '"live_audit_covers_text_selection_and_copy_for_text_page_only"'
        )
    return '"minimal test fixture"'


def write_progress_manifest(root: Path, evidence: tuple[str, ...]) -> None:
    evidence_json = ",".join(f'"{item}"' for item in evidence)
    write_text(
        root / "docs/storybook-77ui-interaction-manifest.json",
        "{"
        '"schema_version":1,'
        '"policy":{'
        '"harness_scope":"core_public_api_harness",'
        '"uses_storybook_only_state":false,'
        '"uses_inspector_only_change":false,'
        '"uses_preset_label_only_change":false,'
        '"bypasses_core_public_api":false'
        "},"
        '"ui":[{'
        '"page":"progress-bar",'
        '"group":"atoms",'
        '"engine":"layout_alignment",'
        '"public_props_options":["storybook_ui_option_contract::options_for_page"],'
        '"state":["progress state"],'
        '"action":["progress action"],'
        '"event":["progress event"],'
        '"callback":["core callback"],'
        '"required_operations":["pointer"],'
        '"tests":{'
        '"window_interaction":["shared:window_interaction::required_page_tests"],'
        '"visual_interaction":["shared:visual_interaction_progress_bar"],'
        '"guard":["shared:guard"]'
        "},"
        '"audit_status":"verified",'
        f'"evidence":[{evidence_json}],'
        '"gaps":[]'
        "}]"
        "}\n",
    )


def write_leaf_change(root: Path, change: str) -> None:
    write_text(root / f"openspec/changes/{change}/.openspec.yaml", "schema: spec-driven\n")
    write_text(root / f"openspec/changes/{change}/proposal.md", "# Proposal\n")
    write_text(root / f"openspec/changes/{change}/tasks.md", "# Tasks\n")
    write_text(root / f"openspec/changes/{change}/specs/{change}/spec.md", "## ADDED Requirements\n")


def option_contract_source(option_arm: str) -> str:
    return (
        "struct StorybookUiOptionContract;\n"
        "impl StorybookUiOptionContract { fn new(_: &str, _: &str, _: &str) -> Self { Self } }\n"
        "fn options_for_page(page: &str) { match page { "
        '"text" => &TEXT_OPTIONS, '
        f"{option_arm} _ => &[] }} }}\n"
        "const TEXT_OPTIONS: [StorybookUiOptionContract; 4] = ["
        'StorybookUiOptionContract::new("a", "0", "1"),'
        'StorybookUiOptionContract::new("b", "0", "1"),'
        'StorybookUiOptionContract::new("c", "0", "1"),'
        'StorybookUiOptionContract::new("d", "0", "1"),];\n'
        "const BUTTON_OPTIONS: [StorybookUiOptionContract; 4] = ["
        'StorybookUiOptionContract::new("a", "0", "1"),'
        'StorybookUiOptionContract::new("b", "0", "1"),'
        'StorybookUiOptionContract::new("c", "0", "1"),'
        'StorybookUiOptionContract::new("d", "0", "1"),];\n'
    )


if __name__ == "__main__":
    unittest.main()
