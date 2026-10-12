use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::core::terminal_ui::{
    render_interactive_problem_view, TerminalUiProblemView, TerminalUiSection,
};
use crate::core::ui::Ui;
use super::problem::EjudgeProblem;

pub struct EjudgeUI;

impl Ui for EjudgeUI {
    type Input = EjudgeProblem;

    fn render(&self, problem: &Self::Input) {
        let title = problem
            .problem_configuration
            .full_problem_name_localized
            .clone()
            .or_else(|| problem.problem_configuration.full_problem_name_english.clone())
            .or_else(|| problem.problem_configuration.internal_problem_identifier.clone())
            .unwrap_or_else(|| "EJUDGE Problem".to_string());

        let mut summary_statistics = Vec::new();

        if let Some(tl) = problem.problem_configuration.time_limit_seconds {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Time Limit:             ", Style::default().fg(Color::White)),
                Span::styled(format!("{:.2} s", tl), Style::default().fg(Color::Green)),
            ]));
        }
        if let Some(ml) = problem.problem_configuration.maximum_virtual_memory_bytes {
            let raw_s = problem
                .problem_configuration
                .maximum_virtual_memory_raw
                .as_deref()
                .unwrap_or("");
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Max Virtual Memory:     ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.2} MB ({} bytes, raw: '{}')", ml as f64 / (1024.0 * 1024.0), ml, raw_s),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }
        if let Some(ref rev) = problem.problem_configuration.revision_number {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Package Revision:       ", Style::default().fg(Color::White)),
                Span::styled(rev.clone(), Style::default().fg(Color::LightMagenta)),
            ]));
        }
        if let Some(ref extid) = problem.problem_configuration.external_system_identifier {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • External ID (Polygon):  ", Style::default().fg(Color::White)),
                Span::styled(extid.clone(), Style::default().fg(Color::LightCyan)),
            ]));
        }
        summary_statistics.push(Line::from(vec![
            Span::styled("  • Standard Streams:       ", Style::default().fg(Color::White)),
            Span::styled(
                format!(
                    "STDIN: {} │ STDOUT: {} │ Testlib Mode: {}",
                    problem.problem_configuration.use_standard_input.unwrap_or(true),
                    problem.problem_configuration.use_standard_output.unwrap_or(true),
                    problem.problem_configuration.enable_testlib_mode.unwrap_or(false)
                ),
                Style::default().fg(Color::Yellow),
            ),
        ]));
        summary_statistics.push(Line::from(vec![
            Span::styled("  • Total Test Cases:       ", Style::default().fg(Color::White)),
            Span::styled(
                problem.common_summary.total_test_cases_count.to_string(),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    Total Valuer Groups: ", Style::default().fg(Color::White)),
            Span::styled(
                problem.valuer_test_groups.len().to_string(),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]));

        let mut sections = Vec::new();

        // Section: Configuration & Commands
        let mut config_lines = Vec::new();
        if let Some(ref ru) = problem.problem_configuration.full_problem_name_localized {
            config_lines.push(Line::from(vec![
                Span::styled("  Long Name (RU):           ", Style::default().fg(Color::DarkGray)),
                Span::styled(ru.clone(), Style::default().fg(Color::White)),
            ]));
        }
        if let Some(ref en) = problem.problem_configuration.full_problem_name_english {
            config_lines.push(Line::from(vec![
                Span::styled("  Long Name (EN):           ", Style::default().fg(Color::DarkGray)),
                Span::styled(en.clone(), Style::default().fg(Color::White)),
            ]));
        }
        if let Some(ref int_name) = problem.problem_configuration.internal_problem_identifier {
            config_lines.push(Line::from(vec![
                Span::styled("  Internal Identifier:      ", Style::default().fg(Color::DarkGray)),
                Span::styled(int_name.clone(), Style::default().fg(Color::Cyan)),
            ]));
        }
        if let Some(ref tp) = problem.problem_configuration.test_file_pattern {
            config_lines.push(Line::from(vec![
                Span::styled("  Test File Pattern:        ", Style::default().fg(Color::DarkGray)),
                Span::styled(tp.clone(), Style::default().fg(Color::LightYellow)),
            ]));
        }
        if let Some(ref cp) = problem.problem_configuration.correct_answer_file_pattern {
            config_lines.push(Line::from(vec![
                Span::styled("  Answer File Pattern:      ", Style::default().fg(Color::DarkGray)),
                Span::styled(cp.clone(), Style::default().fg(Color::LightYellow)),
            ]));
        }
        if let Some(ref chk) = problem.problem_configuration.checker_command {
            config_lines.push(Line::from(vec![
                Span::styled("  Checker Executable/Cmd:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(chk.clone(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]));
        }
        if let Some(ref sol) = problem.problem_configuration.solution_command {
            config_lines.push(Line::from(vec![
                Span::styled("  Solution Executable/Cmd:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(sol.clone(), Style::default().fg(Color::LightGreen)),
            ]));
        }
        sections.push(TerminalUiSection {
            section_title: "Problem Configuration (problem.cfg)".to_string(),
            lines: config_lines,
        });

        // Section: Valuer Settings & Groups
        if let Some(ref global) = problem.valuer_global_settings {
            let mut val_global_lines = Vec::new();
            val_global_lines.push(Line::from(vec![
                Span::styled("  Report Stats to Judges:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(global.report_statistics_to_judges.to_string(), Style::default().fg(Color::White)),
                Span::styled("    Report Stats to Users: ", Style::default().fg(Color::DarkGray)),
                Span::styled(global.report_statistics_to_users.to_string(), Style::default().fg(Color::White)),
            ]));
            sections.push(TerminalUiSection {
                section_title: "Valuer Global Options (valuer.cfg)".to_string(),
                lines: val_global_lines,
            });
        }

        if !problem.valuer_test_groups.len().is_zero() {
            let mut val_group_lines = Vec::new();
            val_group_lines.push(Line::from(vec![
                Span::styled(
                    format!(
                        "  {:<6} {:<12} {:<10} {:<10} {:<15} {:<15}",
                        "Group", "Tests", "Score", "Per-Test", "Requires", "Flags"
                    ),
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED),
                ),
            ]));

            for g in &problem.valuer_test_groups {
                let range_s = match (g.test_range_start, g.test_range_end) {
                    (Some(s), Some(e)) => format!("{}-{}", s, e),
                    _ => "-".to_string(),
                };
                let score_s = g
                    .total_group_score
                    .map(|s| format!("{:.1}", s))
                    .unwrap_or_else(|| "-".to_string());
                let per_test_s = g
                    .single_test_score
                    .map(|s| format!("{:.1}", s))
                    .unwrap_or_else(|| "-".to_string());
                let req_s = if g.required_group_identifiers.is_empty() {
                    "None".to_string()
                } else {
                    g.required_group_identifiers
                        .iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                };
                let mut flags = Vec::new();
                if g.is_offline_evaluation {
                    flags.push("offline");
                }
                if g.sets_marked {
                    flags.push("marked");
                }
                if g.is_skipped {
                    flags.push("skip");
                }
                if g.test_all_cases {
                    flags.push("test_all");
                }
                let flags_s = if flags.is_empty() {
                    "-".to_string()
                } else {
                    flags.join(",")
                };

                val_group_lines.push(Line::from(vec![
                    Span::styled(format!("  {:<6} ", g.group_identifier), Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{:<12} ", range_s), Style::default().fg(Color::White)),
                    Span::styled(format!("{:<10} ", score_s), Style::default().fg(Color::Green)),
                    Span::styled(format!("{:<10} ", per_test_s), Style::default().fg(Color::LightGreen)),
                    Span::styled(format!("{:<15} ", req_s), Style::default().fg(Color::Cyan)),
                    Span::styled(flags_s, Style::default().fg(Color::LightMagenta)),
                ]));
            }

            sections.push(TerminalUiSection {
                section_title: format!("Valuer Evaluation Groups ({})", problem.valuer_test_groups.len()),
                lines: val_group_lines,
            });
        }

        let view = TerminalUiProblemView {
            format_display_name: "EJUDGE (problem.cfg / valuer.cfg)".to_string(),
            problem_title: title,
            summary_statistics,
            sections,
        };

        render_interactive_problem_view(&view);
    }
}

trait IsZero {
    fn is_zero(&self) -> bool;
}

impl IsZero for usize {
    fn is_zero(&self) -> bool {
        *self == 0
    }
}