use std::fs;
use std::path::{Path, PathBuf};

use crate::core::parser::{Parser, ParserError};
use super::problem::{
    EjudgeProblem, EjudgeProblemConfig, EjudgeValuerGlobalSettings, EjudgeValuerGroup,
};

pub struct EjudgeParser;

impl Parser for EjudgeParser {
    type Output = EjudgeProblem;

    fn parse(&self, folder_path: &Path) -> Result<Self::Output, ParserError> {
        let root_dir = if folder_path.is_file() {
            folder_path.parent().unwrap_or(folder_path)
        } else {
            folder_path
        };

        let problem_cfg_path = find_config_file(root_dir, "problem.cfg")?;
        let raw_problem_cfg = fs::read(&problem_cfg_path).map_err(|e| {
            ParserError::IoError(format!(
                "Failed to read 'problem.cfg' at '{}': {}",
                problem_cfg_path.display(),
                e
            ))
        })?;
        let problem_cfg_text = decode_text(&raw_problem_cfg)?;
        let problem_configuration = parse_problem_cfg(&problem_cfg_text)?;

        let valuer_cfg_path = find_optional_config_file(root_dir, "valuer.cfg");
        let (valuer_global_settings, valuer_test_groups) = if let Some(path) = valuer_cfg_path {
            if let Ok(raw_valuer_cfg) = fs::read(&path) {
                if let Ok(valuer_cfg_text) = decode_text(&raw_valuer_cfg) {
                    parse_valuer_cfg(&valuer_cfg_text)?
                } else {
                    (None, Vec::new())
                }
            } else {
                (None, Vec::new())
            }
        } else {
            (None, Vec::new())
        };

        let total_discovered_test_files_count = count_tests_in_tests_dir(root_dir);

        let mut problem = EjudgeProblem {
            problem_configuration,
            valuer_global_settings,
            valuer_test_groups,
            total_discovered_test_files_count,
            common_summary: crate::core::common::CommonProblemSummary {
                problem_title: None,
                problem_identifier: None,
                resource_limits: crate::core::common::ProblemResourceLimits {
                    time_limit_milliseconds: None,
                    memory_limit_bytes: None,
                    write_limit_bytes: None,
                },
                input_stream: crate::core::common::DataStreamSource::StandardInputOutput,
                output_stream: crate::core::common::DataStreamSource::StandardInputOutput,
                checker_description: None,
                interactor_description: None,
                total_test_cases_count: 0,
                total_test_groups_count: 0,
                sample_tests_count: 0,
                test_groups: Vec::new(),
                sample_tests: Vec::new(),
            },
        };
        problem.common_summary = problem.build_common_summary();

        Ok(problem)
    }
}

fn parse_problem_cfg(content: &str) -> Result<EjudgeProblemConfig, ParserError> {
    let mut config = EjudgeProblemConfig::default();

    for line in content.lines() {
        let clean_line = strip_comments(line).trim();
        if clean_line.is_empty() || clean_line.starts_with('[') {
            continue;
        }

        if let Some((raw_key, raw_value)) = clean_line.split_once('=') {
            let key = raw_key.trim();
            let value = unquote(raw_value.trim());

            match key {
                "use_stdin" => config.use_standard_input = Some(value == "1"),
                "use_stdout" => config.use_standard_output = Some(value == "1"),
                "use_corr" => config.use_correct_answer_file = Some(value == "1"),
                "enable_testlib_mode" => config.enable_testlib_mode = Some(value == "1"),
                "time_limit" => config.time_limit_seconds = value.parse::<f64>().ok(),
                "real_time_limit" => config.real_time_limit_seconds = value.parse::<f64>().ok(),
                "max_vm_size" => {
                    config.maximum_virtual_memory_raw = Some(value.to_string());
                    config.maximum_virtual_memory_bytes = parse_memory_size_string(&value);
                }
                "long_name" => config.full_problem_name_localized = Some(value.to_string()),
                "long_name_en" => config.full_problem_name_english = Some(value.to_string()),
                "internal_name" => config.internal_problem_identifier = Some(value.to_string()),
                "test_pat" => config.test_file_pattern = Some(value.to_string()),
                "corr_pat" => config.correct_answer_file_pattern = Some(value.to_string()),
                "check_cmd" => config.checker_command = Some(value.to_string()),
                "solution_cmd" => config.solution_command = Some(value.to_string()),
                "extid" => config.external_system_identifier = Some(value.to_string()),
                "revision" => config.revision_number = Some(value.to_string()),
                _ => {
                    config
                        .custom_properties
                        .insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    Ok(config)
}

fn parse_valuer_cfg(
    content: &str,
) -> Result<(Option<EjudgeValuerGlobalSettings>, Vec<EjudgeValuerGroup>), ParserError> {
    // Strip comments
    let mut cleaned_lines = Vec::new();
    for line in content.lines() {
        let stripped = if let Some(idx) = line.find('#') {
            &line[..idx]
        } else {
            line
        };
        cleaned_lines.push(stripped);
    }
    let full_cleaned = cleaned_lines.join("\n");

    let mut global_settings = None;
    let mut groups = Vec::new();

    // Tokenizer
    let tokens = tokenize(&full_cleaned);
    let mut idx = 0;

    while idx < tokens.len() {
        let token = &tokens[idx];
        if token == "global" {
            idx += 1;
            if idx < tokens.len() && tokens[idx] == "{" {
                idx += 1;
                let mut stat_to_judges = false;
                let mut stat_to_users = false;

                while idx < tokens.len() && tokens[idx] != "}" {
                    let opt = &tokens[idx];
                    idx += 1;
                    match opt.as_str() {
                        "stat_to_judges" => {
                            if idx < tokens.len() && tokens[idx] != ";" && tokens[idx] != "}" {
                                stat_to_judges = tokens[idx] != "0";
                                idx += 1;
                            } else {
                                stat_to_judges = true;
                            }
                            if idx < tokens.len() && tokens[idx] == ";" {
                                idx += 1;
                            }
                        }
                        "stat_to_users" => {
                            if idx < tokens.len() && tokens[idx] != ";" && tokens[idx] != "}" {
                                stat_to_users = tokens[idx] != "0";
                                idx += 1;
                            } else {
                                stat_to_users = true;
                            }
                            if idx < tokens.len() && tokens[idx] == ";" {
                                idx += 1;
                            }
                        }
                        _ => {
                            // Skip until ';' or '}'
                            while idx < tokens.len() && tokens[idx] != ";" && tokens[idx] != "}" {
                                idx += 1;
                            }
                            if idx < tokens.len() && tokens[idx] == ";" {
                                idx += 1;
                            }
                        }
                    }
                }
                if idx < tokens.len() && tokens[idx] == "}" {
                    idx += 1;
                }
                global_settings = Some(EjudgeValuerGlobalSettings {
                    report_statistics_to_judges: stat_to_judges,
                    report_statistics_to_users: stat_to_users,
                });
            }
        } else if token == "group" {
            idx += 1;
            let group_id = if idx < tokens.len() {
                let id = tokens[idx].parse::<u32>().unwrap_or(0);
                idx += 1;
                id
            } else {
                0
            };

            if idx < tokens.len() && tokens[idx] == "{" {
                idx += 1;

                let mut test_range_start = None;
                let mut test_range_end = None;
                let mut total_group_score = None;
                let mut single_test_score = None;
                let mut required_group_identifiers = Vec::new();
                let mut is_offline_evaluation = false;
                let mut sets_marked = false;
                let mut is_skipped = false;
                let mut sets_marked_if_passed_group_identifiers = Vec::new();
                let mut minimum_passed_tests_count = None;
                let mut skip_if_not_rejudge = false;
                let mut report_group_statistics_to_judges = false;
                let mut report_group_statistics_to_users = false;
                let mut user_status_on_pass = None;
                let mut test_all_cases = false;
                let mut zero_score_if_only_tests = Vec::new();

                while idx < tokens.len() && tokens[idx] != "}" {
                    let opt = tokens[idx].clone();
                    idx += 1;

                    match opt.as_str() {
                        "tests" => {
                            if idx < tokens.len() && tokens[idx] != ";" {
                                let spec = &tokens[idx];
                                idx += 1;
                                if let Some((start_s, end_s)) = spec.split_once('-') {
                                    test_range_start = start_s.trim().parse::<u32>().ok();
                                    test_range_end = end_s.trim().parse::<u32>().ok();
                                }
                            }
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "score" => {
                            if idx < tokens.len() && tokens[idx] != ";" {
                                total_group_score = tokens[idx].parse::<f64>().ok();
                                idx += 1;
                            }
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "test_score" => {
                            if idx < tokens.len() && tokens[idx] != ";" {
                                single_test_score = tokens[idx].parse::<f64>().ok();
                                idx += 1;
                            }
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "requires" => {
                            while idx < tokens.len() && tokens[idx] != ";" && tokens[idx] != "}" {
                                for part in tokens[idx].split(',') {
                                    if let Ok(id) = part.trim().parse::<u32>() {
                                        required_group_identifiers.push(id);
                                    }
                                }
                                idx += 1;
                            }
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "offline" => {
                            is_offline_evaluation = true;
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "sets_marked" => {
                            sets_marked = true;
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "skip" => {
                            is_skipped = true;
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "sets_marked_if_passed" => {
                            while idx < tokens.len() && tokens[idx] != ";" && tokens[idx] != "}" {
                                for part in tokens[idx].split(',') {
                                    if let Ok(id) = part.trim().parse::<u32>() {
                                        sets_marked_if_passed_group_identifiers.push(id);
                                    }
                                }
                                idx += 1;
                            }
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "pass_if_count" => {
                            if idx < tokens.len() && tokens[idx] != ";" {
                                minimum_passed_tests_count = tokens[idx].parse::<u32>().ok();
                                idx += 1;
                            }
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "skip_if_not_rejudge" => {
                            skip_if_not_rejudge = true;
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "stat_to_judges" => {
                            report_group_statistics_to_judges = true;
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "stat_to_users" => {
                            report_group_statistics_to_users = true;
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "user_status" => {
                            if idx < tokens.len() && tokens[idx] != ";" {
                                user_status_on_pass = Some(tokens[idx].clone());
                                idx += 1;
                            }
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "test_all" => {
                            test_all_cases = true;
                            consume_semicolon(&tokens, &mut idx);
                        }
                        "0_if" => {
                            while idx < tokens.len() && tokens[idx] != ";" && tokens[idx] != "}" {
                                for part in tokens[idx].split(',') {
                                    if let Ok(id) = part.trim().parse::<u32>() {
                                        zero_score_if_only_tests.push(id);
                                    }
                                }
                                idx += 1;
                            }
                            consume_semicolon(&tokens, &mut idx);
                        }
                        _ => {
                            consume_until_semicolon(&tokens, &mut idx);
                        }
                    }
                }
                if idx < tokens.len() && tokens[idx] == "}" {
                    idx += 1;
                }

                groups.push(EjudgeValuerGroup {
                    group_identifier: group_id,
                    test_range_start,
                    test_range_end,
                    total_group_score,
                    single_test_score,
                    required_group_identifiers,
                    is_offline_evaluation,
                    sets_marked,
                    is_skipped,
                    sets_marked_if_passed_group_identifiers,
                    minimum_passed_tests_count,
                    skip_if_not_rejudge,
                    report_group_statistics_to_judges,
                    report_group_statistics_to_users,
                    user_status_on_pass,
                    test_all_cases,
                    zero_score_if_only_tests,
                });
            }
        } else {
            idx += 1;
        }
    }

    Ok((global_settings, groups))
}

fn tokenize(content: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in content.chars() {
        if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else if ch == '{' || ch == '}' || ch == ';' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            tokens.push(ch.to_string());
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn consume_semicolon(tokens: &[String], idx: &mut usize) {
    if *idx < tokens.len() && tokens[*idx] == ";" {
        *idx += 1;
    }
}

fn consume_until_semicolon(tokens: &[String], idx: &mut usize) {
    while *idx < tokens.len() && tokens[*idx] != ";" && tokens[*idx] != "}" {
        *idx += 1;
    }
    consume_semicolon(tokens, idx);
}

fn strip_comments(line: &str) -> &str {
    let without_hash = if let Some(idx) = line.find('#') {
        &line[..idx]
    } else {
        line
    };
    if let Some(idx) = without_hash.find("//") {
        &without_hash[..idx]
    } else {
        without_hash
    }
}

fn unquote(s: &str) -> &str {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        if trimmed.len() >= 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            ""
        }
    } else {
        trimmed
    }
}

fn parse_memory_size_string(val: &str) -> Option<u64> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return None;
    }

    let last_char = trimmed.chars().last()?;
    if last_char.is_ascii_digit() {
        return trimmed.parse::<u64>().ok();
    }

    let unit = last_char.to_ascii_uppercase();
    let num_str = &trimmed[..trimmed.len() - 1].trim();
    let num = num_str.parse::<u64>().ok()?;

    match unit {
        'B' => Some(num),
        'K' => Some(num * 1024),
        'M' => Some(num * 1024 * 1024),
        'G' => Some(num * 1024 * 1024 * 1024),
        _ => Some(num),
    }
}

fn find_config_file(folder_path: &Path, file_name: &str) -> Result<PathBuf, ParserError> {
    let direct = folder_path.join(file_name);
    if direct.is_file() {
        return Ok(direct);
    }

    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.eq_ignore_ascii_case(file_name))
                    .unwrap_or(false)
            {
                return Ok(path);
            }
        }
    }

    Err(ParserError::FileNotFound(format!(
        "'{}' was not found in directory '{}'",
        file_name,
        folder_path.display()
    )))
}

fn find_optional_config_file(folder_path: &Path, file_name: &str) -> Option<PathBuf> {
    find_config_file(folder_path, file_name).ok()
}

fn count_tests_in_tests_dir(root_dir: &Path) -> usize {
    let tests_dir = root_dir.join("tests");
    if tests_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(tests_dir) {
            return entries.flatten().filter(|e| e.path().is_file()).count();
        }
    }
    0
}

fn decode_text(bytes: &[u8]) -> Result<String, ParserError> {
    if let Ok(valid_utf8) = std::str::from_utf8(bytes) {
        return Ok(valid_utf8.to_string());
    }

    let (decoded, _, had_errors) = encoding_rs::WINDOWS_1251.decode(bytes);
    if !had_errors {
        return Ok(decoded.into_owned());
    }

    let (decoded_lossy, _) = encoding_rs::WINDOWS_1251.decode_without_bom_handling(bytes);
    Ok(decoded_lossy.into_owned())
}