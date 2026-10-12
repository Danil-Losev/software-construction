use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::core::terminal_ui::{
    render_interactive_problem_view, TerminalUiProblemView, TerminalUiSection,
};
use crate::core::ui::Ui;
use super::problem::Pcms2Problem;

pub struct Pcms2UI;

impl Ui for Pcms2UI {
    type Input = Pcms2Problem;

    fn render(&self, problem: &Self::Input) {
        let title = problem
            .localized_names
            .first()
            .map(|n| n.problem_title.clone())
            .or_else(|| problem.short_name.clone())
            .or_else(|| problem.problem_global_identifier.clone())
            .unwrap_or_else(|| "PCMS2 Problem".to_string());

        let mut summary_statistics = Vec::new();

        if let Some(ref rev) = problem.revision_number {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Revision:               ", Style::default().fg(Color::White)),
                Span::styled(rev.clone(), Style::default().fg(Color::LightMagenta)),
            ]));
        }
        if let Some(ref ver) = problem.specification_version {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Package Version:        ", Style::default().fg(Color::White)),
                Span::styled(ver.clone(), Style::default().fg(Color::LightMagenta)),
            ]));
        }
        if let Some(ref id) = problem.problem_global_identifier {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Global Identifier:      ", Style::default().fg(Color::White)),
                Span::styled(id.clone(), Style::default().fg(Color::LightCyan)),
            ]));
        }
        if let Some(ref short) = problem.short_name {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Short Name:             ", Style::default().fg(Color::White)),
                Span::styled(short.clone(), Style::default().fg(Color::LightYellow)),
            ]));
        }
        if let Some(ref cpu) = problem.cpu_name {
            let speed = problem
                .cpu_speed_mhz
                .map(|s| format!(" @ {} MHz", s))
                .unwrap_or_default();
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Benchmark CPU:          ", Style::default().fg(Color::White)),
                Span::styled(format!("{}{}", cpu, speed), Style::default().fg(Color::DarkGray)),
            ]));
        }
        if let Some(tl) = problem.common_summary.resource_limits.time_limit_milliseconds {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Time Limit:             ", Style::default().fg(Color::White)),
                Span::styled(format!("{} ms ({:.2} s)", tl, tl as f64 / 1000.0), Style::default().fg(Color::Green)),
            ]));
        }
        if let Some(ml) = problem.common_summary.resource_limits.memory_limit_bytes {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Memory Limit:           ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.2} MB ({} bytes)", ml as f64 / (1024.0 * 1024.0), ml),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }
        summary_statistics.push(Line::from(vec![
            Span::styled("  • Total Tests:            ", Style::default().fg(Color::White)),
            Span::styled(
                problem.common_summary.total_test_cases_count.to_string(),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    Test Groups: ", Style::default().fg(Color::White)),
            Span::styled(
                problem.common_summary.total_test_groups_count.to_string(),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    Sample Tests: ", Style::default().fg(Color::White)),
            Span::styled(
                problem.common_summary.sample_tests_count.to_string(),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]));

        let mut sections = Vec::new();

        // Section: Localized Names & Statement Documents
        let mut name_docs_lines = Vec::new();
        for name in &problem.localized_names {
            name_docs_lines.push(Line::from(vec![
                Span::styled(format!("  Title [{}]: ", name.language), Style::default().fg(Color::DarkGray)),
                Span::styled(name.problem_title.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]));
        }
        for stmt in &problem.statement_documents {
            let lang = stmt.language.as_deref().unwrap_or("default");
            let mime = stmt.mime_type.as_deref().unwrap_or("document");
            name_docs_lines.push(Line::from(vec![
                Span::styled("  Statement Document:       ", Style::default().fg(Color::DarkGray)),
                Span::styled(stmt.file_path.clone(), Style::default().fg(Color::Cyan)),
                Span::styled(format!(" (Lang: {}, Type: {})", lang, mime), Style::default().fg(Color::DarkGray)),
            ]));
        }
        for tut in &problem.tutorial_documents {
            name_docs_lines.push(Line::from(vec![
                Span::styled("  Tutorial Solution:        ", Style::default().fg(Color::DarkGray)),
                Span::styled(tut.file_path.clone(), Style::default().fg(Color::LightGreen)),
            ]));
        }
        if !name_docs_lines.is_empty() {
            sections.push(TerminalUiSection {
                section_title: "Problem Names & Statement Documents".to_string(),
                lines: name_docs_lines,
            });
        }

        // Section: Tools, Checkers & Solutions
        let mut asset_lines = Vec::new();
        if let Some(ref chk) = problem.checker {
            let chk_name = chk.checker_name.as_deref().unwrap_or("Custom Checker");
            let bin_path = chk
                .executable
                .as_ref()
                .and_then(|e| e.binary_path.as_deref())
                .unwrap_or("-");
            let src_path = chk
                .executable
                .as_ref()
                .and_then(|e| e.source_path.as_deref())
                .unwrap_or("-");
            asset_lines.push(Line::from(vec![
                Span::styled("  Checker:                  ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} (Binary: {}, Source: {})", chk_name, bin_path, src_path), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]));
        }
        if let Some(ref inter) = problem.interactor {
            let bin = inter.binary_path.as_deref().unwrap_or("-");
            let src = inter.source_path.as_deref().unwrap_or("-");
            asset_lines.push(Line::from(vec![
                Span::styled("  Interactor:               ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("Binary: {}, Source: {}", bin, src), Style::default().fg(Color::Magenta)),
            ]));
        }
        for val in &problem.validators {
            let bin = val.binary_path.as_deref().unwrap_or("-");
            let src = val.source_path.as_deref().unwrap_or("-");
            asset_lines.push(Line::from(vec![
                Span::styled("  Validator:                ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("Binary: {}, Source: {}", bin, src), Style::default().fg(Color::LightRed)),
            ]));
        }
        for sol in &problem.solutions {
            let bin = sol.executable.binary_path.as_deref().unwrap_or("-");
            let src = sol.executable.source_path.as_deref().unwrap_or("-");
            asset_lines.push(Line::from(vec![
                Span::styled("  Solution:                 ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:<15} ", sol.verdict_tag), Style::default().fg(Color::Yellow)),
                Span::styled(format!("Source: {}, Binary: {}", src, bin), Style::default().fg(Color::White)),
            ]));
        }
        if !asset_lines.is_empty() {
            sections.push(TerminalUiSection {
                section_title: "Verification Tools, Checkers & Solutions".to_string(),
                lines: asset_lines,
            });
        }

        // Section: Testsets & Test Groups
        let mut testset_lines = Vec::new();
        for set in &problem.test_sets {
            testset_lines.push(Line::from(vec![
                Span::styled(format!("  [Testset: '{}']", set.testset_name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!(
                        " Pattern: In='{}', Ans='{}'",
                        set.input_path_pattern.as_deref().unwrap_or("-"),
                        set.answer_path_pattern.as_deref().unwrap_or("-")
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));

            if !set.test_groups.is_empty() {
                testset_lines.push(Line::from(vec![
                    Span::styled(
                        format!(
                            "    {:<25} {:<10} {:<10} {:<10} {:<15}",
                            "Group/Subtask", "Bonus", "Score", "Feedback", "Requires"
                        ),
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED),
                    ),
                ]));

                for group in &set.test_groups {
                    let name_s = group
                        .comment_description
                        .as_deref()
                        .or(group.group_identifier.as_deref())
                        .unwrap_or("Subtask");
                    let bonus_s = group
                        .group_bonus_points
                        .map(|b| format!("{:.1}", b))
                        .unwrap_or_else(|| "-".to_string());
                    let score_s = group.scoring_strategy.as_deref().unwrap_or("sum");
                    let feed_s = group.feedback_strategy.as_deref().unwrap_or("-");
                    let req_s = if group.required_group_identifiers.is_empty() {
                        "None".to_string()
                    } else {
                        group.required_group_identifiers.join(", ")
                    };

                    testset_lines.push(Line::from(vec![
                        Span::styled(format!("    {:<25} ", name_s), Style::default().fg(Color::Yellow)),
                        Span::styled(format!("{:<10} ", bonus_s), Style::default().fg(Color::Green)),
                        Span::styled(format!("{:<10} ", score_s), Style::default().fg(Color::White)),
                        Span::styled(format!("{:<10} ", feed_s), Style::default().fg(Color::LightCyan)),
                        Span::styled(req_s, Style::default().fg(Color::LightMagenta)),
                    ]));
                }
            }
            testset_lines.push(Line::from(""));
        }
        if !testset_lines.is_empty() {
            sections.push(TerminalUiSection {
                section_title: "Test Sets & Subtask Hierarchy".to_string(),
                lines: testset_lines,
            });
        }

        // Section: Sample Tests & Standalone Tests
        let mut sample_lines = Vec::new();
        for set in &problem.test_sets {
            for (idx, test) in set.standalone_test_cases.iter().enumerate() {
                if test.is_sample_test || idx < 10 {
                    let sample_badge = if test.is_sample_test { "[SAMPLE] " } else { "" };
                    let group_s = test.group_identifier.as_deref().unwrap_or("-");
                    let cmd_s = test.generation_command.as_deref().unwrap_or("manual");
                    sample_lines.push(Line::from(vec![
                        Span::styled(format!("  {:>3}. ", idx + 1), Style::default().fg(Color::DarkGray)),
                        Span::styled(sample_badge, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("Group: {:<5} │ Cmd/Method: {}", group_s, cmd_s), Style::default().fg(Color::White)),
                    ]));
                }
            }
        }
        if !sample_lines.is_empty() {
            sections.push(TerminalUiSection {
                section_title: "Sample Tests & Test Definitions Preview".to_string(),
                lines: sample_lines,
            });
        }

        let view = TerminalUiProblemView {
            format_display_name: "PCMS2 (problem.xml)".to_string(),
            problem_title: title,
            summary_statistics,
            sections,
        };

        render_interactive_problem_view(&view);
    }
}