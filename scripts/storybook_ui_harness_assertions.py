from __future__ import annotations

import json
from pathlib import Path

from storybook_ui_harness_interaction_assertions import StorybookUiInteractionHarness
from storybook_ui_harness_ledger import StorybookUiHarnessLedger
from storybook_ui_harness_manifest import StorybookUiHarnessManifest
from storybook_ui_harness_public_options import PublicOptionRequirements
from storybook_ui_harness_sources import StorybookUiHarnessSources
from storybook_manual_acceptance_queue import manual_acceptance_queue

MIN_PRESET_COUNT = 4
MIN_OPTION_COUNT = 4


class StorybookUiHarness:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.sources = StorybookUiHarnessSources(root)

    def failures(self) -> list[str]:
        required = self.sources.required_pages()
        interactive_runtime_pages = self.sources.interactive_runtime_pages()
        preset_counts = self.sources.preset_counts()
        option_pages = self.sources.option_pages()
        option_counts = self.sources.option_counts()
        menu_pages = self.sources.menu_pages()
        leaf_changes = self.sources.leaf_changes()
        leaf_statuses = self.sources.leaf_statuses()
        dedicated_pages = self.sources.dedicated_pages()
        priority_order = self.sources.priority_order()
        failures: list[str] = []
        failures.extend(self.missing_presets(required, preset_counts))
        failures.extend(self.missing_options(required, option_pages))
        failures.extend(self.low_option_counts(required, option_pages, option_counts))
        failures.extend(self.low_preset_counts(required, preset_counts))
        failures.extend(
            self.option_preset_count_failures(
                required, option_pages, option_counts, preset_counts
            )
        )
        failures.extend(self.menu_mismatch_failures(required, menu_pages))
        failures.extend(self.leaf_change_failures(menu_pages, leaf_changes))
        failures.extend(self.dedicated_page_failures(leaf_statuses, dedicated_pages))
        failures.extend(self.split_summary_count_failures(leaf_statuses))
        failures.extend(self.priority_order_failures(menu_pages, leaf_changes, priority_order))
        failures.extend(
            self.interactive_runtime_page_failures(
                interactive_runtime_pages,
                menu_pages,
                preset_counts,
            )
        )
        failures.extend(StorybookUiInteractionHarness(self.root).failures())
        failures.extend(self.text_input_runtime_state_failures())
        failures.extend(self.text_area_runtime_state_failures())
        failures.extend(self.preset_tab_label_layout_failures())
        failures.extend(self.public_option_contract_failures())
        failures.extend(self.manual_acceptance_queue_failures())
        failures.extend(StorybookUiHarnessLedger(self.root).failures())
        failures.extend(StorybookUiHarnessManifest(self.root, required).failures())
        if "storybook_ui_option_contract::settings_rows_for" not in self.sources.read(
            "crates/katana-ui-core-storybook/src/visual/inspector_rows.rs"
        ):
            failures.append("Inspector settings must render Storybook UI option contract rows")
        return failures

    def interactive_runtime_page_failures(
        self,
        runtime_pages: list[str],
        menu_pages: list[str],
        preset_counts: dict[str, int],
    ) -> list[str]:
        failures: list[str] = []
        manifest_pages = self.interaction_manifest_pages()
        for page in runtime_pages:
            if page in menu_pages:
                failures.append(f"{page}: interactive runtime page must not appear in the Canvas menu")
            if page in preset_counts:
                failures.append(
                    f"{page}: interactive runtime page must not use Canvas preset labels"
                )
            if page in manifest_pages:
                failures.append(
                    f"{page}: interactive runtime page must not appear in the Canvas manifest"
                )
            runtime_contracts = {
                "command-chrome": self.command_chrome_runtime_failures,
                "text-command-root": self.text_command_root_runtime_failures,
            }
            contract = runtime_contracts.get(page)
            if contract is None:
                failures.append(f"{page}: interactive runtime page has no strict runtime contract")
                continue
            failures.extend(contract())
        return failures

    def interaction_manifest_pages(self) -> set[str]:
        manifest = self.root / "docs/storybook-77ui-interaction-manifest.json"
        if not manifest.exists():
            return set()
        try:
            payload = json.loads(manifest.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            return set()
        entries = payload.get("ui") if isinstance(payload, dict) else None
        if not isinstance(entries, list):
            return set()
        return {
            page
            for entry in entries
            if isinstance(entry, dict)
            if isinstance((page := entry.get("page")), str)
        }

    def command_chrome_runtime_failures(self) -> list[str]:
        required_tokens = {
            "crates/katana-ui-core-storybook/src/lib.rs": ("interactive_runtime_pages",),
            "crates/katana-ui-core-storybook/src/visual/window.rs": (
                "command_chrome_runtime::handles_page",
                "command_chrome_runtime::open_window",
            ),
            "crates/katana-ui-core-storybook/src/visual/command_chrome_runtime.rs": (
                "CommandChromeStorybookApp",
                "eframe::run_native",
            ),
            "crates/katana-ui-core-storybook/src/visual/command_chrome_app.rs": (
                "CommandChromeStorybookApp",
                "CommandChromeSurface",
            ),
            "crates/katana-ui-core-storybook/src/visual/command_chrome_surface.rs": (
                "show_command_chrome",
                "EguiCommandChromeAdapter",
            ),
            "crates/katana-ui-core-storybook/src/visual/command_chrome_script.rs": (
                "run_scripted_sequence",
            ),
        }
        failures: list[str] = []
        for path, tokens in required_tokens.items():
            source_path = self.root / path
            source = source_path.read_text(encoding="utf-8") if source_path.exists() else ""
            for token in tokens:
                if token not in source:
                    failures.append(f"command-chrome: runtime contract missing `{token}` in {path}")
        visual_root = self.root / "crates/katana-ui-core-storybook/src/visual"
        # The adapter's ArtifactCanvasBounds is an actual-frame fact, not a
        # Storybook canvas renderer. Reject renderer/fallback implementations.
        forbidden = ("egui::Canvas", "minifb", "fallback", "StorybookFallbackRenderer")
        for path in sorted(visual_root.glob("command_chrome_*.rs")):
            source = path.read_text(encoding="utf-8")
            for token in forbidden:
                if token in source:
                    failures.append(
                        f"command-chrome: runtime source must not contain `{token}`: {path.relative_to(self.root)}"
                    )
        return failures

    def text_command_root_runtime_failures(self) -> list[str]:
        required_tokens = {
            "crates/katana-ui-core-storybook/src/lib.rs": (
                "interactive_runtime_pages",
            ),
            "crates/katana-ui-core-storybook/src/requirements.rs": (
                '"text-command-root"',
            ),
            "crates/katana-ui-core-storybook/src/visual/window.rs": (
                "text_command_root_storybook::handles_page",
                "text_command_root_storybook::open_window",
            ),
            "crates/katana-ui-core-storybook/src/visual/text_command_root_storybook.rs": (
                "TextCommandRootStorybookApp",
                "eframe::run_native",
                "EguiTextCommandSurfaceHostRoot",
                "EguiTextCommandSurfaceHostProjectionEncoder::token",
                "EguiTextCommandSurfaceRootFactory::default()",
                ".retain(token)",
                "root.show(ui)",
                ".forward_events_once(&mut forwarder)",
                "consumed_once: receipt.consumed_once()",
                "forwarder_calls: forwarder.calls",
                "if sequence.steps.len() < 9",
                "write_mp4",
                "decode_mp4",
                '"framemd5"',
                "decoded_frame_count != sequence.steps.len()",
                '"text-command-root-manifest.json"',
                "FullRootManifest::from_sequence",
                "event_receipt: EventReceiptEvidence",
                "frame_sequence_sha256",
                "decoder: DecoderEvidence",
                "encoder_capability_verified",
                "muxer_capability_verified",
            ),
            "crates/katana-ui-core/src/egui/text_command_surface/host_root/factory_api.rs": (
                "pub fn retain(",
            ),
            "crates/katana-ui-core/src/egui/text_command_surface/host_root/token_api.rs": (
                "pub fn token(",
            ),
            "crates/katana-ui-core/src/egui/text_command_surface/host_root/types.rs": (
                "pub struct EguiTextCommandSurfacePresentationToken",
                "pub struct EguiTextCommandSurfaceHostProjectionEncoder",
            ),
            "crates/katana-ui-core/tests/egui_host_root_facade_contract.rs": (
                "opaque_tokens_and_transport_have_no_clone_or_serialize_derives",
                "compatibility_types_are_hidden",
            ),
            "crates/katana-ui-core-storybook/src/visual/text_command_root_storybook_tests.rs": (
                "full_root_storybook_uses_only_the_public_facade_root",
            ),
            "crates/katana-ui-core/tests/egui_text_command_root_contract.rs": (
                "root_event_batch_forwards_once_and_returns_a_closed_receipt",
                "assert!(receipt.consumed_once())",
            ),
        }
        failures: list[str] = []
        for path, tokens in required_tokens.items():
            source_path = self.root / path
            source = source_path.read_text(encoding="utf-8") if source_path.exists() else ""
            if path.endswith("text_command_root_storybook.rs"):
                source = source.split("#[cfg(test)]", 1)[0]
            for token in tokens:
                if token not in source:
                    failures.append(
                        f"text-command-root: runtime contract missing `{token}` in {path}"
                    )
        return failures

    def manual_acceptance_queue_failures(self) -> list[str]:
        script = self.root / "scripts/storybook_manual_acceptance_queue.py"
        test = self.root / "scripts/test_storybook_manual_acceptance_queue.py"
        smoke = self.root / "scripts/storybook_manual_acceptance_smoke.py"
        smoke_test = self.root / "scripts/test_storybook_manual_acceptance_smoke.py"
        failures: list[str] = []
        if not script.exists():
            failures.append("manual acceptance queue script is missing")
        if not test.exists():
            failures.append("manual acceptance queue test is missing")
        if not smoke.exists():
            failures.append("manual acceptance smoke script is missing")
        elif not self.manual_acceptance_smoke_writes_evidence(smoke):
            failures.append(
                "manual acceptance smoke must write storybook-manual-acceptance-evidence.json"
            )
        if not smoke_test.exists():
            failures.append("manual acceptance smoke test is missing")
        justfile_path = self.root / "Justfile"
        if not justfile_path.exists():
            failures.append("Justfile: storybook-manual-acceptance-smoke recipe is missing")
        else:
            justfile = self.sources.read("Justfile")
            if "storybook-manual-acceptance-smoke:" not in justfile:
                failures.append("Justfile: storybook-manual-acceptance-smoke recipe is missing")
            if "scripts/storybook_manual_acceptance_smoke.py" not in justfile:
                failures.append(
                    "Justfile: storybook-manual-acceptance-smoke must run the smoke script"
                )
            if "--headless-interaction-audit" not in justfile:
                failures.append(
                    "Justfile: storybook-manual-acceptance-smoke must regenerate live interaction audit"
                )
            regression_lines = [
                line.strip()
                for line in justfile.splitlines()
                if line.strip().startswith("storybook-regression:")
            ]
            if not any("storybook-manual-acceptance-smoke" in line for line in regression_lines):
                failures.append(
                    "Justfile: storybook-regression must include storybook-manual-acceptance-smoke"
                )
        manifest = self.root / "docs/storybook-77ui-interaction-manifest.json"
        if manifest.exists():
            try:
                queue = manual_acceptance_queue(manifest)
            except ValueError as error:
                failures.append(str(error))
                queue = []
            for entry in queue:
                page = entry.get("page")
                command = entry.get("command")
                if not isinstance(page, str) or f"--open-window {page}" not in str(command):
                    failures.append(f"{page}: manual acceptance queue command is invalid")
                    continue
                smoke_command = str(entry.get("smoke_command"))
                expected_smoke_suffix = f"--open-window {entry.get('minimum_observation_frames')} {page}"
                if expected_smoke_suffix not in smoke_command:
                    failures.append(
                        f"{page}: manual acceptance smoke command must include `{expected_smoke_suffix}`"
                    )
                observations = entry.get("acceptance_observations")
                if not isinstance(observations, list) or not observations:
                    failures.append(
                        f"{page}: manual acceptance queue must include observation checklist"
                    )
                if page == "text":
                    checks = entry.get("acceptance_checks")
                    if not isinstance(checks, list) or "text_drag_selection" not in checks:
                        failures.append(
                            "text: manual acceptance queue must include text_drag_selection"
                        )
                    if not isinstance(checks, list) or "text_keyboard_copy" not in checks:
                        failures.append(
                            "text: manual acceptance queue must include text_keyboard_copy"
                        )
                    if (
                        not isinstance(checks, list)
                        or "text_zero_distance_drag_no_selection" not in checks
                    ):
                        failures.append(
                            "text: manual acceptance queue must include text_zero_distance_drag_no_selection"
                        )
                if page == "progress-bar":
                    if "--open-window 48 progress-bar" not in smoke_command:
                        failures.append(
                            "progress-bar: manual acceptance queue smoke command must open 48 frames"
                        )
                    if entry.get("minimum_observation_frames") != 48:
                        failures.append(
                            "progress-bar: manual acceptance queue must require 48 observation frames"
                        )
                    checks = entry.get("acceptance_checks")
                    if not isinstance(checks, list) or "progress_preview_click" not in checks:
                        failures.append(
                            "progress-bar: manual acceptance queue must include progress_preview_click"
                        )
                    if not isinstance(checks, list) or "progress_timed_cycle" not in checks:
                        failures.append(
                            "progress-bar: manual acceptance queue must include progress_timed_cycle"
                        )
                    if (
                        not isinstance(checks, list)
                        or "progress_indeterminate_segment_motion" not in checks
                    ):
                        failures.append(
                            "progress-bar: manual acceptance queue must include progress_indeterminate_segment_motion"
                        )
        return failures

    def manual_acceptance_smoke_writes_evidence(self, smoke: Path) -> bool:
        source = smoke.read_text(encoding="utf-8")
        return (
            "storybook-manual-acceptance-evidence.json" in source
            and "manual_acceptance_evidence_report" in source
            and "write_evidence_report" in source
            and '"command": entry.get("command")' in source
            and '"smoke_command": entry.get("smoke_command")' in source
            and '"minimum_observation_frames": entry.get("minimum_observation_frames")'
            in source
        )

    @staticmethod
    def menu_mismatch_failures(required: list[str], menu_pages: list[str]) -> list[str]:
        required_set = set(required)
        menu_set = set(menu_pages)
        failures = [
            f"{page}: required page missing from Storybook menu"
            for page in sorted(required_set - menu_set)
        ]
        failures.extend(
            f"{page}: Storybook menu page missing from requirements.rs"
            for page in sorted(menu_set - required_set)
        )
        return failures

    def leaf_change_failures(self, menu_pages: list[str], leaf_changes: dict[str, str]) -> list[str]:
        if not leaf_changes:
            return ["storybook menu leaf change split document is missing or empty"]
        menu_set = set(menu_pages)
        leaf_set = set(leaf_changes)
        failures = [
            f"{page}: Storybook menu page missing leaf change"
            for page in sorted(menu_set - leaf_set)
        ]
        failures.extend(
            f"{page}: leaf change exists without Storybook menu page"
            for page in sorted(leaf_set - menu_set)
        )
        for page in sorted(menu_set & leaf_set):
            failures.extend(self.missing_leaf_artifacts(page, leaf_changes[page]))
        return failures

    def missing_leaf_artifacts(self, page: str, change: str) -> list[str]:
        change_root = self.root / "openspec/changes" / change
        expected = [
            change_root / ".openspec.yaml",
            change_root / "proposal.md",
            change_root / "tasks.md",
            change_root / "specs" / change / "spec.md",
        ]
        return [
            f"{page}: leaf change `{change}` missing {path.relative_to(self.root)}"
            for path in expected
            if not path.exists()
        ]

    @staticmethod
    def dedicated_page_failures(leaf_statuses: dict[str, str], dedicated_pages: set[str]) -> list[str]:
        failures = [
            f"{page}: split table says page-specific rendering exists, but draw_page has no branch"
            for page, status in sorted(leaf_statuses.items())
            if status == "page別描画あり" and page not in dedicated_pages
        ]
        failures.extend(
            f"{page}: draw_page has a branch, but split table does not mark page-specific rendering"
            for page in sorted(dedicated_pages)
            if leaf_statuses.get(page) == "page別描画未作成"
        )
        return failures

    def split_summary_count_failures(self, leaf_statuses: dict[str, str]) -> list[str]:
        summary_counts = self.sources.split_summary_counts()
        if not summary_counts:
            return ["storybook menu split summary counts are missing"]
        expected = {
            "あり": sum(1 for status in leaf_statuses.values() if status == "page別描画あり"),
            "未作成": sum(1 for status in leaf_statuses.values() if status == "page別描画未作成"),
        }
        return [
            f"storybook menu split summary count `{label}` must be {count}, got {summary_counts.get(label)}"
            for label, count in expected.items()
            if summary_counts.get(label) != count
        ]

    @staticmethod
    def priority_order_failures(
        menu_pages: list[str],
        leaf_changes: dict[str, str],
        priority_order: list[tuple[str, str, str]],
    ) -> list[str]:
        if not priority_order:
            return ["storybook menu priority order document is missing or empty"]
        failures: list[str] = []
        priorities = [priority for priority, _page, _change in priority_order]
        pages = [page for _priority, page, _change in priority_order]
        changes = [change for _priority, _page, change in priority_order]
        failures.extend(StorybookUiHarness.duplicate_failures("priority", priorities))
        failures.extend(StorybookUiHarness.duplicate_failures("priority page", pages))
        failures.extend(StorybookUiHarness.duplicate_failures("priority leaf change", changes))
        failures.extend(StorybookUiHarness.priority_gap_failures(priorities))
        failures.extend(StorybookUiHarness.missing_priority_failures(menu_pages, pages))
        for _priority, page, change in priority_order:
            if leaf_changes.get(page) != change:
                failures.append(f"{page}: priority leaf change `{change}` does not match split table")
        return failures

    @staticmethod
    def missing_priority_failures(menu_pages: list[str], pages: list[str]) -> list[str]:
        menu_set = set(menu_pages)
        priority_pages = set(pages)
        failures = [
            f"{page}: Storybook menu page missing priority number"
            for page in sorted(menu_set - priority_pages)
        ]
        failures.extend(
            f"{page}: priority number exists without Storybook menu page"
            for page in sorted(priority_pages - menu_set)
        )
        return failures

    @staticmethod
    def duplicate_failures(label: str, values: list[str]) -> list[str]:
        seen: set[str] = set()
        duplicates: set[str] = set()
        for value in values:
            if value in seen:
                duplicates.add(value)
            seen.add(value)
        return [f"duplicate {label}: {value}" for value in sorted(duplicates)]

    @staticmethod
    def priority_gap_failures(priorities: list[str]) -> list[str]:
        expected = [f"SB-{index:03}" for index in range(1, len(priorities) + 1)]
        if sorted(priorities) == expected:
            return []
        return ["storybook menu priority order must be contiguous from SB-001"]

    @staticmethod
    def missing_presets(required: list[str], preset_counts: dict[str, int]) -> list[str]:
        return [f"{page}: missing explicit Storybook preset labels" for page in required if page not in preset_counts]

    @staticmethod
    def missing_options(required: list[str], option_pages: dict[str, str]) -> list[str]:
        return [f"{page}: missing Storybook UI option contract" for page in required if page not in option_pages]

    @staticmethod
    def low_option_counts(
        required: list[str],
        option_pages: dict[str, str],
        option_counts: dict[str, int],
    ) -> list[str]:
        failures = []
        for page in required:
            target = option_pages.get(page)
            if target is not None and option_counts.get(target, 0) < MIN_OPTION_COUNT:
                failures.append(f"{page}: Storybook option contract must cover at least {MIN_OPTION_COUNT} options")
        return failures

    @staticmethod
    def low_preset_counts(required: list[str], preset_counts: dict[str, int]) -> list[str]:
        return [
            f"{page}: Storybook presets must expose at least {MIN_PRESET_COUNT} tabs"
            for page in required
            if preset_counts.get(page, 0) < MIN_PRESET_COUNT
        ]

    @staticmethod
    def option_preset_count_failures(
        required: list[str],
        option_pages: dict[str, str],
        option_counts: dict[str, int],
        preset_counts: dict[str, int],
    ) -> list[str]:
        failures: list[str] = []
        for page in required:
            target = option_pages.get(page)
            option_count = option_counts.get(target, 0) if target is not None else 0
            preset_count = preset_counts.get(page, 0)
            if option_count > 0 and preset_count < option_count:
                failures.append(
                    f"{page}: Storybook preset tabs must cover every option contract "
                    f"({preset_count} presets < {option_count} options)"
                )
        return failures

    def text_input_runtime_state_failures(self) -> list[str]:
        screen_state = self.read_optional(
            "crates/katana-ui-core-storybook/src/visual/screen_state.rs"
        )
        runtime_state = self.read_optional(
            "crates/katana-ui-core-storybook/src/visual/text_input_screen_state.rs"
        )
        interaction_state = self.read_optional(
            "crates/katana-ui-core-storybook/src/visual/screen_state_text_input.rs"
        )
        if not screen_state and not runtime_state and not interaction_state:
            return []
        failures: list[str] = []
        forbidden = (
            "text_input_state:",
            "text_input_uses_live_value:",
            "text_input_caret_visible:",
        )
        for token in forbidden:
            if token in screen_state:
                failures.append(f"text-input Storybook state must be instance-scoped, not `{token}`")
        required = (
            "TextInputStateStore",
            "BTreeMap<&'static str, TextInputRuntimeState>",
        )
        for token in required:
            if token not in runtime_state:
                failures.append(f"text-input Storybook runtime state missing instance store token: {token}")
        if "register_text_input_readonly_block" not in interaction_state:
            failures.append("text-input Storybook keyboard path must block readonly mutation")
        return failures

    def text_area_runtime_state_failures(self) -> list[str]:
        screen_state = self.read_optional(
            "crates/katana-ui-core-storybook/src/visual/screen_state.rs"
        )
        runtime_state = self.read_optional(
            "crates/katana-ui-core-storybook/src/visual/text_area_screen_state.rs"
        )
        interaction_state = self.read_optional(
            "crates/katana-ui-core-storybook/src/visual/screen_state_text_area.rs"
        )
        query_state = self.read_optional(
            "crates/katana-ui-core-storybook/src/visual/screen_state_text_area_queries.rs"
        )
        scroll_state = self.read_optional(
            "crates/katana-ui-core-storybook/src/visual/screen_state_text_area_scroll.rs"
        )
        if not screen_state and not runtime_state and not interaction_state and not scroll_state:
            return []
        failures: list[str] = []
        forbidden = (
            "text_area_value:",
            "text_area_focused:",
            "text_area_uses_live_value:",
            "text_area_caret_visible:",
            "text_area_wrap_enabled:",
            "text_area_resize_enabled:",
            "text_area_vertical_scroll_enabled:",
            "text_area_horizontal_scroll_enabled:",
            "text_area_scroll_offset:",
            "text_area_scroll_x_offset:",
            "text_area_resize_width_delta:",
            "text_area_resize_height_delta:",
        )
        for token in forbidden:
            if token in screen_state:
                failures.append(f"text-area Storybook state must be instance-scoped, not `{token}`")
        required = (
            "TextAreaStateStore",
            "BTreeMap<&'static str, TextAreaRuntimeState>",
            "DEFAULT_TEXT_AREA_INSTANCE",
        )
        for token in required:
            if token not in runtime_state:
                failures.append(f"text-area Storybook runtime state missing instance store token: {token}")
        if "text_area_runtime_mut_for(" not in interaction_state + scroll_state:
            failures.append("text-area Storybook keyboard and scroll paths must mutate runtime store")
        if "text_area_runtime_for(" not in interaction_state + query_state:
            failures.append("text-area Storybook preview state must read runtime store")
        return failures

    def preset_tab_label_layout_failures(self) -> list[str]:
        tab_source = self.read_optional("crates/katana-ui-core-storybook/src/visual/preset_tabs.rs")
        label_source = self.read_optional(
            "crates/katana-ui-core-storybook/src/visual/preset_tab_label.rs"
        )
        if not tab_source and not label_source:
            return []
        contracts = (
            ("tab", tab_source, ("preset_tab_label::fit", "with_clip")),
            (
                "label",
                label_source,
                ("measure_width", "TRUNCATION_MARKER", "measured_width_for_test"),
            ),
        )
        failures: list[str] = []
        for label, source, tokens in contracts:
            if not source:
                failures.append(f"preset tab {label} label layout contract file is missing")
                continue
            failures.extend(
                f"preset tab labels must be measured and clipped inside each tab: missing {token}"
                for token in tokens
                if token not in source
            )
        return failures

    def public_option_contract_failures(self) -> list[str]:
        settings_by_page = self.sources.option_settings_by_page()
        failures: list[str] = []
        for requirement in PublicOptionRequirements.all():
            source = self.read_optional(requirement.source)
            if requirement.source_token not in source:
                continue
            settings = settings_by_page.get(requirement.page, set())
            if requirement.setting not in settings:
                failures.append(
                    f"{requirement.page}: public option `{requirement.source_token}` "
                    f"missing Storybook Inspector option `{requirement.setting}`"
                )
        return failures

    def read_optional(self, relative: str) -> str:
        path = self.root / relative
        if not path.exists():
            return ""
        return path.read_text(encoding="utf-8")
