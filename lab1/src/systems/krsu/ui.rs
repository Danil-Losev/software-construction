use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::core::terminal_ui::{
    render_interactive_problem_view, TerminalUiProblemView, TerminalUiSection,
};
use crate::core::ui::Ui;
use super::problem::KrsuProblem;

pub struct KrsuUI;

impl Ui for KrsuUI {
    type Input = KrsuProblem;

    fn render(&self, problem: &Self::Input) {
        let problem_title = problem
            .problem_statement_file_path
            .clone()
            .unwrap_or_else(|| "KRSU Contest Problem".to_string());

        let mut summary_statistics = Vec::new();
        if let Some(tl) = problem.time_limit_milliseconds {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Time Limit:             ", Style::default().fg(Color::White)),
                Span::styled(format!("{} ms ({:.2} s)", tl, tl as f64 / 1000.0), Style::default().fg(Color::Green)),
            ]));
        }
        if let Some(ml) = problem.memory_limit_bytes {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Memory Limit:           ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{} bytes ({:.2} MB)", ml, ml as f64 / (1024.0 * 1024.0)),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }
        if let Some(ref run_type) = problem.run_evaluation_type {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Run Evaluation Mode:    ", Style::default().fg(Color::White)),
                Span::styled(run_type.description(), Style::default().fg(Color::LightBlue)),
            ]));
        }
        if let Some(v) = problem.package_test_version {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Package Test Version:   ", Style::default().fg(Color::White)),
                Span::styled(v.to_string(), Style::default().fg(Color::LightMagenta)),
            ]));
        }
        summary_statistics.push(Line::from(vec![
            Span::styled("  • Total Test Cases:       ", Style::default().fg(Color::White)),
            Span::styled(problem.test_cases.len().to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("    Total Test Groups: ", Style::default().fg(Color::White)),
            Span::styled(problem.test_groups.len().to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));

        let mut sections = Vec::new();

        // Section: System Executables & Problem Statement
        let mut exec_lines = Vec::new();
        if let Some(ref chk) = problem.checker_executable {
            exec_lines.push(Line::from(vec![
                Span::styled("  Checker Program:          ", Style::default().fg(Color::DarkGray)),
                Span::styled(chk.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
        }
        if let Some(ref inter) = problem.interactor_executable {
            exec_lines.push(Line::from(vec![
                Span::styled("  Interactor Program:       ", Style::default().fg(Color::DarkGray)),
                Span::styled(inter.clone(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]));
        }
        if let Some(ref stmt) = problem.problem_statement_file_path {
            exec_lines.push(Line::from(vec![
                Span::styled("  Problem Statement File:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(stmt.clone(), Style::default().fg(Color::White)),
            ]));
        }
        if !exec_lines.is_empty() {
            sections.push(TerminalUiSection { table: None,
                section_title: "Problem Statement & Verification Tools".to_string(),
                lines: exec_lines,
            });
        }

        // Section: Test Groups
        if !problem.test_groups.is_empty() {
            let mut group_lines = Vec::new();
            group_lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<10} {:<15} {:<30}", "Group ID", "Score Points", "Prerequisites"),
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED),
                ),
            ]));
            for group in &problem.test_groups {
                let prereqs = if group.prerequisite_group_identifiers.is_empty() {
                    "None".to_string()
                } else {
                    group.prerequisite_group_identifiers.join(", ")
                };
                group_lines.push(Line::from(vec![
                    Span::styled(format!("  {:<10} ", group.group_identifier), Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{:<15.1} ", group.score_points), Style::default().fg(Color::Green)),
                    Span::styled(prereqs, Style::default().fg(Color::LightCyan)),
                ]));
            }
            sections.push(TerminalUiSection { table: None,
                section_title: format!("Test Groups Specification ({})", problem.test_groups.len()),
                lines: group_lines,
            });
        }

        // Section: Test Cases
        let mut test_lines = Vec::new();
        test_lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<6} {:<20} {:<20} {:<10} {:<10}", "#", "Input File", "Output File", "Group", "Points"),
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED),
            ),
        ]));
        for (idx, test) in problem.test_cases.iter().enumerate() {
            let group_s = test.group_identifier.as_deref().unwrap_or("-");
            let points_s = test
                .score_points
                .map(|p| format!("{:.1}", p))
                .unwrap_or_else(|| "-".to_string());
            test_lines.push(Line::from(vec![
                Span::styled(format!("  [{:>3}] ", idx + 1), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:<20} ", test.input_file_path), Style::default().fg(Color::White)),
                Span::styled(format!("{:<20} ", test.output_file_path), Style::default().fg(Color::White)),
                Span::styled(format!("{:<10} ", group_s), Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:<10}", points_s), Style::default().fg(Color::Green)),
            ]));
        }
        sections.push(TerminalUiSection { table: None,
            section_title: format!("Individual Test Cases ({})", problem.test_cases.len()),
            lines: test_lines,
        });

        let view = TerminalUiProblemView {
            format_display_name: "KRSU (testinfo.xml)".to_string(),
            problem_title,
            summary_statistics,
            sections,
        };

        render_interactive_problem_view(&view);
    }
}