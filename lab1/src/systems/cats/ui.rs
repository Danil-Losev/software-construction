use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::core::terminal_ui::{
    render_interactive_problem_view, TerminalUiProblemView, TerminalUiSection,
};
use crate::core::ui::Ui;
use super::problem::CatsProblem;

pub struct CatsUI;

impl Ui for CatsUI {
    type Input = CatsProblem;

    fn render(&self, problem: &Self::Input) {
        let mut summary_statistics = Vec::new();

        if let Some(tlimit) = problem.time_limit_seconds {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Time Limit:             ", Style::default().fg(Color::White)),
                Span::styled(format!("{:.2} s", tlimit), Style::default().fg(Color::Green)),
            ]));
        }
        if let Some(mlimit) = problem.memory_limit_bytes {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Memory Limit:           ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.2} MB ({} bytes)", mlimit as f64 / (1024.0 * 1024.0), mlimit),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }
        if let Some(wlimit) = problem.write_limit_bytes {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Write Limit:            ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.2} MB", wlimit as f64 / (1024.0 * 1024.0)),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }
        if let Some(ref author) = problem.author_name {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Author:                 ", Style::default().fg(Color::White)),
                Span::styled(author.clone(), Style::default().fg(Color::LightMagenta)),
            ]));
        }
        if !problem.supported_natural_languages.is_empty() {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Languages:              ", Style::default().fg(Color::White)),
                Span::styled(problem.supported_natural_languages.join(", "), Style::default().fg(Color::LightCyan)),
            ]));
        }
        if let Some(diff) = problem.author_difficulty_rating {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Author Difficulty:      ", Style::default().fg(Color::White)),
                Span::styled(format!("{}/100", diff), Style::default().fg(Color::Yellow)),
            ]));
        }
        summary_statistics.push(Line::from(vec![
            Span::styled("  • Input Stream:           ", Style::default().fg(Color::White)),
            Span::styled(
                problem.input_file_name.as_deref().unwrap_or("*STDIN").to_string(),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled("    Output Stream: ", Style::default().fg(Color::White)),
            Span::styled(
                problem.output_file_name.as_deref().unwrap_or("*STDOUT").to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]));
        summary_statistics.push(Line::from(vec![
            Span::styled("  • Total Discovered Tests: ", Style::default().fg(Color::White)),
            Span::styled(
                problem.common_summary.total_test_cases_count.to_string(),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    Sample Tests: ", Style::default().fg(Color::White)),
            Span::styled(
                problem.sample_demonstrations.len().to_string(),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]));

        let mut sections = Vec::new();

        // Section: Problem Statement
        if let Some(ref stmt) = problem.problem_statement {
            let mut stmt_lines = Vec::new();
            for line in stmt.content_stml.lines() {
                stmt_lines.push(Line::from(format!("  {}", line)));
            }
            sections.push(TerminalUiSection {
                section_title: "Problem Statement & Description".to_string(),
                lines: stmt_lines,
            });
        }

        // Section: Constraints & Format specifications
        let mut format_lines = Vec::new();
        if let Some(ref constraints) = problem.problem_constraints_description {
            format_lines.push(Line::from(vec![
                Span::styled("  [Constraints]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            for l in constraints.lines() {
                format_lines.push(Line::from(format!("    {}", l)));
            }
            format_lines.push(Line::from(""));
        }
        if let Some(ref inp_fmt) = problem.input_format_description {
            format_lines.push(Line::from(vec![
                Span::styled("  [Input Format]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
            for l in inp_fmt.lines() {
                format_lines.push(Line::from(format!("    {}", l)));
            }
            format_lines.push(Line::from(""));
        }
        if let Some(ref out_fmt) = problem.output_format_description {
            format_lines.push(Line::from(vec![
                Span::styled("  [Output Format]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
            for l in out_fmt.lines() {
                format_lines.push(Line::from(format!("    {}", l)));
            }
        }
        if !format_lines.is_empty() {
            sections.push(TerminalUiSection {
                section_title: "Input/Output Specifications & Constraints".to_string(),
                lines: format_lines,
            });
        }

        // Section: Sample Demonstrations
        if !problem.sample_demonstrations.is_empty() {
            let mut sample_lines = Vec::new();
            for (idx, sample) in problem.sample_demonstrations.iter().enumerate() {
                sample_lines.push(Line::from(vec![
                    Span::styled(format!("  Sample #{} (Rank {}):", idx + 1, sample.rank_specification), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]));

                sample_lines.push(Line::from(vec![
                    Span::styled("    Input:", Style::default().fg(Color::DarkGray)),
                ]));
                if let Some(ref in_txt) = sample.input_inline_text {
                    for l in in_txt.lines() {
                        sample_lines.push(Line::from(vec![
                            Span::styled(format!("      {}", l), Style::default().fg(Color::White)),
                        ]));
                    }
                } else if let Some(ref in_src) = sample.input_file_path {
                    sample_lines.push(Line::from(format!("      [File: {}]", in_src)));
                }

                sample_lines.push(Line::from(vec![
                    Span::styled("    Output:", Style::default().fg(Color::DarkGray)),
                ]));
                if let Some(ref out_txt) = sample.output_inline_text {
                    for l in out_txt.lines() {
                        sample_lines.push(Line::from(vec![
                            Span::styled(format!("      {}", l), Style::default().fg(Color::Green)),
                        ]));
                    }
                } else if let Some(ref out_src) = sample.output_file_path {
                    sample_lines.push(Line::from(format!("      [File: {}]", out_src)));
                }
                sample_lines.push(Line::from(""));
            }
            sections.push(TerminalUiSection {
                section_title: format!("Sample Demonstrations ({})", problem.sample_demonstrations.len()),
                lines: sample_lines,
            });
        }

        // Section: Executables, Checkers & Solutions
        let mut tool_lines = Vec::new();
        if let Some(ref std_chk) = problem.standard_checker_identifier {
            tool_lines.push(Line::from(vec![
                Span::styled("  Standard Checker:         ", Style::default().fg(Color::DarkGray)),
                Span::styled(std_chk.clone(), Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)),
            ]));
        }
        for chk in &problem.custom_checkers {
            let env_str = chk.development_environment.human_name.as_deref().unwrap_or("auto");
            tool_lines.push(Line::from(vec![
                Span::styled("  Custom Checker:           ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} [{}]", chk.checker_name, chk.source_file_path), Style::default().fg(Color::Cyan)),
                Span::styled(format!(" (Env: {})", env_str), Style::default().fg(Color::DarkGray)),
            ]));
        }
        for sol in &problem.reference_solutions {
            tool_lines.push(Line::from(vec![
                Span::styled("  Reference Solution:       ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} [{}]", sol.solution_name, sol.source_file_path), Style::default().fg(Color::Green)),
            ]));
        }
        for generator in &problem.test_generators {
            let pat = generator.output_file_pattern.as_deref().unwrap_or("-");
            tool_lines.push(Line::from(vec![
                Span::styled("  Test Generator:           ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} [{}] Pattern: {}", generator.generator_name, generator.source_file_path, pat), Style::default().fg(Color::Yellow)),
            ]));
        }
        for val in &problem.test_validators {
            tool_lines.push(Line::from(vec![
                Span::styled("  Test Validator:           ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} [{}]", val.validator_name, val.source_file_path), Style::default().fg(Color::LightRed)),
            ]));
        }
        if let Some(ref inter) = problem.interactor {
            tool_lines.push(Line::from(vec![
                Span::styled("  Interactor / Controller:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} [{}]", inter.interactor_name, inter.source_file_path), Style::default().fg(Color::Magenta)),
            ]));
        }
        if !tool_lines.is_empty() {
            sections.push(TerminalUiSection {
                section_title: "Executables, Checkers & Solutions".to_string(),
                lines: tool_lines,
            });
        }

        // Section: Test Ranges & Test Sets
        let mut test_spec_lines = Vec::new();
        for range in &problem.test_ranges {
            let in_desc = range
                .input_definition
                .as_ref()
                .and_then(|i| i.source_file_path.clone())
                .unwrap_or_else(|| "Generator".to_string());
            let out_desc = range
                .output_definition
                .as_ref()
                .and_then(|o| o.solution_reference_name.clone().map(|s| format!("solution: {}", s)))
                .unwrap_or_else(|| "File".to_string());
            test_spec_lines.push(Line::from(vec![
                Span::styled(format!("  Test Range #{}-#{}: ", range.range_start_number, range.range_end_number), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("Input: {} │ Output: {}", in_desc, out_desc), Style::default().fg(Color::White)),
            ]));
        }
        for set in &problem.test_sets {
            test_spec_lines.push(Line::from(vec![
                Span::styled(format!("  Test Set '{}': ", set.testset_name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("Tests: {} │ Points: {:?}", set.test_rank_specification, set.total_score_points), Style::default().fg(Color::White)),
            ]));
        }
        if !test_spec_lines.is_empty() {
            sections.push(TerminalUiSection {
                section_title: "Test Ranges & Test Sets".to_string(),
                lines: test_spec_lines,
            });
        }

        // Section: Assets, Keywords & Imports
        let mut asset_lines = Vec::new();
        if !problem.keyword_codes.is_empty() {
            asset_lines.push(Line::from(vec![
                Span::styled("  Keywords:                 ", Style::default().fg(Color::DarkGray)),
                Span::styled(problem.keyword_codes.join(", "), Style::default().fg(Color::Yellow)),
            ]));
        }
        for pic in &problem.pictures {
            asset_lines.push(Line::from(vec![
                Span::styled("  Embedded Picture:         ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} (src: {})", pic.name, pic.source_path), Style::default().fg(Color::LightBlue)),
            ]));
        }
        for imp in &problem.imported_dependencies {
            let imp_type = imp.imported_object_type.as_deref().unwrap_or("module");
            asset_lines.push(Line::from(vec![
                Span::styled("  Imported Dependency:      ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} (type: {})", imp.global_unique_identifier, imp_type), Style::default().fg(Color::LightGreen)),
            ]));
        }
        if !asset_lines.is_empty() {
            sections.push(TerminalUiSection {
                section_title: "Assets, Imports & Keywords".to_string(),
                lines: asset_lines,
            });
        }

        let view = TerminalUiProblemView {
            format_display_name: format!("CATS v{}", problem.cats_specification_version),
            problem_title: problem.problem_title.clone(),
            summary_statistics,
            sections,
        };

        render_interactive_problem_view(&view);
    }
}