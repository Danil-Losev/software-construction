use std::fs;
use std::path::{Path, PathBuf};

use crate::core::parser::{Parser, ParserError};
use super::problem::{KrsuProblem, KrsuRunEvaluationType, KrsuTestCase, KrsuTestGroup};

pub struct KrsuParser;

impl Parser for KrsuParser {
    type Output = KrsuProblem;

    fn parse(&self, folder_path: &Path) -> Result<Self::Output, ParserError> {
        let xml_path = find_krsu_xml_file(folder_path)?;
        let raw_bytes = fs::read(&xml_path).map_err(|e| {
            ParserError::IoError(format!(
                "Failed to read file '{}': {}",
                xml_path.display(),
                e
            ))
        })?;

        let xml_text = decode_text(&raw_bytes)?;
        let document = roxmltree::Document::parse(&xml_text).map_err(|e| {
            ParserError::XmlError(format!(
                "XML parsing error in '{}': {}",
                xml_path.display(),
                e
            ))
        })?;

        let root_node = document.root_element();
        if root_node.tag_name().name() != "testinfo" {
            return Err(ParserError::InvalidSyntax(format!(
                "Expected root tag <testinfo>, but found <{}> in '{}'",
                root_node.tag_name().name(),
                xml_path.display()
            )));
        }

        let mut checker_executable = None;
        let mut interactor_executable = None;
        let mut problem_statement_file_path = None;
        let mut memory_limit_bytes = None;
        let mut time_limit_milliseconds = None;
        let mut package_test_version = None;
        let mut run_evaluation_type = None;
        let mut test_groups = Vec::new();
        let mut test_cases = Vec::new();

        for child in root_node.children() {
            if !child.is_element() {
                continue;
            }

            match child.tag_name().name() {
                "checker" => {
                    checker_executable = child.text().map(|s| s.trim().to_string());
                }
                "interactor" => {
                    interactor_executable = child.text().map(|s| s.trim().to_string());
                }
                "problem" => {
                    problem_statement_file_path = child.text().map(|s| s.trim().to_string());
                }
                "memorylimit" => {
                    if let Some(text) = child.text() {
                        memory_limit_bytes = text.trim().parse::<u64>().ok();
                    }
                }
                "timelimit" => {
                    if let Some(text) = child.text() {
                        time_limit_milliseconds = text.trim().parse::<u64>().ok();
                    }
                }
                "testversion" => {
                    if let Some(text) = child.text() {
                        package_test_version = text.trim().parse::<u32>().ok();
                    }
                }
                "runtype" => {
                    if let Some(text) = child.text() {
                        if let Ok(val) = text.trim().parse::<i32>() {
                            run_evaluation_type = Some(KrsuRunEvaluationType::from_raw(val));
                        }
                    }
                }
                "group" => {
                    let group_identifier = child.attribute("id").unwrap_or_default().to_string();
                    let score_points = child
                        .attribute("points")
                        .and_then(|p| p.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let prerequisite_string = child.attribute("prereq").unwrap_or_default();
                    let prerequisite_group_identifiers = if prerequisite_string.trim().is_empty() {
                        Vec::new()
                    } else {
                        prerequisite_string
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    };

                    test_groups.push(KrsuTestGroup {
                        group_identifier,
                        score_points,
                        prerequisite_group_identifiers,
                    });
                }
                "test" => {
                    let input_file_path = child.attribute("input").unwrap_or_default().to_string();
                    let output_file_path = child.attribute("output").unwrap_or_default().to_string();
                    let group_identifier = child
                        .attribute("groupid")
                        .map(|g| g.trim().to_string())
                        .filter(|g| !g.is_empty());
                    let score_points = child.attribute("points").and_then(|p| p.parse::<f64>().ok());

                    test_cases.push(KrsuTestCase {
                        input_file_path,
                        output_file_path,
                        group_identifier,
                        score_points,
                    });
                }
                _ => {}
            }
        }

        let mut problem = KrsuProblem {
            checker_executable,
            interactor_executable,
            problem_statement_file_path,
            memory_limit_bytes,
            time_limit_milliseconds,
            package_test_version,
            run_evaluation_type,
            test_groups,
            test_cases,
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

fn find_krsu_xml_file(folder_path: &Path) -> Result<PathBuf, ParserError> {
    if folder_path.is_file() {
        return Ok(folder_path.to_path_buf());
    }

    let direct_path = folder_path.join("testinfo.xml");
    if direct_path.is_file() {
        return Ok(direct_path);
    }

    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.eq_ignore_ascii_case("testinfo.xml"))
                    .unwrap_or(false)
            {
                return Ok(path);
            }
        }
    }

    Err(ParserError::FileNotFound(format!(
        "testinfo.xml was not found in directory '{}'",
        folder_path.display()
    )))
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