use std::fs;
use std::path::{Path, PathBuf};

use crate::core::parser::{Parser, ParserError};
use super::problem::{
    Pcms2Checker, Pcms2DocumentReference, Pcms2Executable, Pcms2Problem, Pcms2ProblemName,
    Pcms2ResourceFile, Pcms2Solution, Pcms2TestCase, Pcms2TestGroup, Pcms2TestSet,
};

pub struct Pcms2Parser;

impl Parser for Pcms2Parser {
    type Output = Pcms2Problem;

    fn parse(&self, folder_path: &Path) -> Result<Self::Output, ParserError> {
        let xml_path = find_pcms2_xml_file(folder_path)?;
        let raw_bytes = fs::read(&xml_path).map_err(|e| {
            ParserError::IoError(format!(
                "Failed to read PCMS2 file '{}': {}",
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
        if root_node.tag_name().name() != "problem" {
            return Err(ParserError::InvalidSyntax(format!(
                "Expected root element <problem>, found <{}> in '{}'",
                root_node.tag_name().name(),
                xml_path.display()
            )));
        }

        let specification_version = root_node.attribute("version").map(|s| s.to_string());
        let problem_global_identifier = root_node.attribute("id").map(|s| s.to_string());
        let short_name = root_node.attribute("short-name").map(|s| s.to_string());
        let revision_number = root_node.attribute("revision").map(|s| s.to_string());
        let polygon_url = root_node.attribute("url").map(|s| s.to_string());

        let mut localized_names = Vec::new();
        let mut statement_documents = Vec::new();
        let mut tutorial_documents = Vec::new();
        let mut cpu_name = None;
        let mut cpu_speed_mhz = None;
        let mut default_input_file_name = None;
        let mut default_output_file_name = None;
        let mut judging_script_type = None;
        let mut test_sets = Vec::new();
        let mut resource_files = Vec::new();
        let mut executable_tools = Vec::new();
        let mut checker = None;
        let mut interactor = None;
        let mut validators = Vec::new();
        let mut solutions = Vec::new();
        let mut general_documents = Vec::new();
        let mut tags = Vec::new();

        for child in root_node.children().filter(|c| c.is_element()) {
            match child.tag_name().name() {
                "names" => {
                    for name_child in child.children().filter(|c| c.is_element()) {
                        if name_child.tag_name().name() == "name" {
                            let language = name_child
                                .attribute("language")
                                .unwrap_or("default")
                                .to_string();
                            let problem_title =
                                name_child.attribute("value").unwrap_or_default().to_string();
                            localized_names.push(Pcms2ProblemName {
                                language,
                                problem_title,
                            });
                        }
                    }
                }
                "statements" => {
                    for stmt_child in child.children().filter(|c| c.is_element()) {
                        if stmt_child.tag_name().name() == "statement" {
                            statement_documents.push(parse_document_reference(&stmt_child));
                        }
                    }
                }
                "tutorials" => {
                    for tut_child in child.children().filter(|c| c.is_element()) {
                        if tut_child.tag_name().name() == "tutorial" {
                            tutorial_documents.push(parse_document_reference(&tut_child));
                        }
                    }
                }
                "judging" => {
                    cpu_name = child.attribute("cpu-name").map(|s| s.to_string());
                    cpu_speed_mhz = child
                        .attribute("cpu-speed")
                        .and_then(|s| s.parse::<u32>().ok());
                    default_input_file_name = child.attribute("input-file").map(|s| s.to_string());
                    default_output_file_name =
                        child.attribute("output-file").map(|s| s.to_string());

                    for judging_child in child.children().filter(|c| c.is_element()) {
                        match judging_child.tag_name().name() {
                            "script" => {
                                judging_script_type =
                                    judging_child.attribute("type").map(|s| s.to_string());
                                for script_child in
                                    judging_child.children().filter(|c| c.is_element())
                                {
                                    match script_child.tag_name().name() {
                                        "testset" => {
                                            test_sets.push(parse_testset_node(&script_child));
                                        }
                                        "verifier" => {
                                            if let Some(bin) = script_child
                                                .children()
                                                .find(|c| c.is_element() && c.tag_name().name() == "binary")
                                            {
                                                checker = Some(Pcms2Checker {
                                                    checker_name: None,
                                                    checker_type: script_child
                                                        .attribute("type")
                                                        .map(|s| s.to_string()),
                                                    executable: Some(Pcms2Executable {
                                                        source_path: None,
                                                        source_type: None,
                                                        binary_path: bin
                                                            .attribute("file")
                                                            .map(|s| s.to_string()),
                                                        binary_type: bin
                                                            .attribute("executable-id")
                                                            .map(|s| s.to_string()),
                                                        executable_identifier: bin
                                                            .attribute("executable-id")
                                                            .map(|s| s.to_string()),
                                                    }),
                                                });
                                            }
                                        }
                                        "interactor" => {
                                            if let Some(bin) = script_child
                                                .children()
                                                .find(|c| c.is_element() && c.tag_name().name() == "binary")
                                            {
                                                interactor = Some(Pcms2Executable {
                                                    source_path: None,
                                                    source_type: None,
                                                    binary_path: bin
                                                        .attribute("file")
                                                        .map(|s| s.to_string()),
                                                    binary_type: bin
                                                        .attribute("executable-id")
                                                        .map(|s| s.to_string()),
                                                    executable_identifier: bin
                                                        .attribute("executable-id")
                                                        .map(|s| s.to_string()),
                                                });
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            "testset" => {
                                test_sets.push(parse_testset_node(&judging_child));
                            }
                            _ => {}
                        }
                    }
                }
                "files" => {
                    for files_child in child.children().filter(|c| c.is_element()) {
                        match files_child.tag_name().name() {
                            "resources" => {
                                for res_child in files_child.children().filter(|c| c.is_element()) {
                                    if res_child.tag_name().name() == "file" {
                                        resource_files.push(Pcms2ResourceFile {
                                            file_path: res_child
                                                .attribute("path")
                                                .unwrap_or_default()
                                                .to_string(),
                                            file_type: res_child
                                                .attribute("type")
                                                .map(|s| s.to_string()),
                                        });
                                    }
                                }
                            }
                            "executables" => {
                                for exec_child in files_child.children().filter(|c| c.is_element())
                                {
                                    if exec_child.tag_name().name() == "executable" {
                                        executable_tools.push(parse_executable_node(&exec_child));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "assets" => {
                    for assets_child in child.children().filter(|c| c.is_element()) {
                        match assets_child.tag_name().name() {
                            "checker" => {
                                let checker_name =
                                    assets_child.attribute("name").map(|s| s.to_string());
                                let checker_type =
                                    assets_child.attribute("type").map(|s| s.to_string());
                                let executable = Some(parse_executable_node(&assets_child));
                                checker = Some(Pcms2Checker {
                                    checker_name,
                                    checker_type,
                                    executable,
                                });
                            }
                            "validators" => {
                                for val_child in assets_child.children().filter(|c| c.is_element())
                                {
                                    if val_child.tag_name().name() == "validator" {
                                        validators.push(parse_executable_node(&val_child));
                                    }
                                }
                            }
                            "solutions" => {
                                for sol_child in assets_child.children().filter(|c| c.is_element())
                                {
                                    if sol_child.tag_name().name() == "solution" {
                                        let verdict_tag = sol_child
                                            .attribute("tag")
                                            .unwrap_or("unknown")
                                            .to_string();
                                        let executable = parse_executable_node(&sol_child);
                                        solutions.push(Pcms2Solution {
                                            verdict_tag,
                                            executable,
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "documents" => {
                    for doc_child in child.children().filter(|c| c.is_element()) {
                        if doc_child.tag_name().name() == "document" {
                            general_documents.push(parse_document_reference(&doc_child));
                        }
                    }
                }
                "tags" => {
                    for tag_child in child.children().filter(|c| c.is_element()) {
                        if tag_child.tag_name().name() == "tag" {
                            if let Some(val) = tag_child.attribute("value") {
                                tags.push(val.to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        let mut problem = Pcms2Problem {
            specification_version,
            problem_global_identifier,
            short_name,
            revision_number,
            polygon_url,
            localized_names,
            statement_documents,
            tutorial_documents,
            cpu_name,
            cpu_speed_mhz,
            default_input_file_name,
            default_output_file_name,
            judging_script_type,
            test_sets,
            resource_files,
            executable_tools,
            checker,
            interactor,
            validators,
            solutions,
            general_documents,
            tags,
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

fn parse_document_reference(node: &roxmltree::Node<'_, '_>) -> Pcms2DocumentReference {
    Pcms2DocumentReference {
        character_set: node.attribute("charset").map(|s| s.to_string()),
        language: node.attribute("language").map(|s| s.to_string()),
        file_path: node.attribute("path").unwrap_or_default().to_string(),
        mime_type: node.attribute("type").map(|s| s.to_string()),
    }
}

fn parse_executable_node(node: &roxmltree::Node<'_, '_>) -> Pcms2Executable {
    let mut source_path = None;
    let mut source_type = None;
    let mut binary_path = None;
    let mut binary_type = None;

    for child in node.children().filter(|c| c.is_element()) {
        match child.tag_name().name() {
            "source" => {
                source_path = child.attribute("path").map(|s| s.to_string());
                source_type = child.attribute("type").map(|s| s.to_string());
            }
            "binary" => {
                binary_path = child
                    .attribute("path")
                    .or_else(|| child.attribute("file"))
                    .map(|s| s.to_string());
                binary_type = child
                    .attribute("type")
                    .or_else(|| child.attribute("executable-id"))
                    .map(|s| s.to_string());
            }
            _ => {}
        }
    }

    Pcms2Executable {
        source_path,
        source_type,
        binary_path,
        binary_type,
        executable_identifier: None,
    }
}

fn parse_testset_node(node: &roxmltree::Node<'_, '_>) -> Pcms2TestSet {
    let testset_name = node.attribute("name").unwrap_or("tests").to_string();
    let mut time_limit_milliseconds = node.attribute("time-limit").and_then(parse_time_limit_str);
    let mut memory_limit_bytes = node
        .attribute("memory-limit")
        .and_then(parse_memory_limit_str);
    let mut test_count = node
        .attribute("test-count")
        .and_then(|s| s.parse::<usize>().ok());
    let input_file_name = node.attribute("input-name").map(|s| s.to_string());
    let output_file_name = node.attribute("output-name").map(|s| s.to_string());
    let mut input_path_pattern = node
        .attribute("input-href")
        .or_else(|| node.attribute("input-path-pattern"))
        .map(|s| s.to_string());
    let mut answer_path_pattern = node
        .attribute("answer-href")
        .or_else(|| node.attribute("answer-path-pattern"))
        .map(|s| s.to_string());

    let mut test_groups = Vec::new();
    let mut standalone_test_cases = Vec::new();

    for child in node.children().filter(|c| c.is_element()) {
        match child.tag_name().name() {
            "time-limit" => {
                if let Some(txt) = child.text() {
                    time_limit_milliseconds = parse_time_limit_str(txt);
                }
            }
            "memory-limit" => {
                if let Some(txt) = child.text() {
                    memory_limit_bytes = parse_memory_limit_str(txt);
                }
            }
            "test-count" => {
                if let Some(txt) = child.text() {
                    test_count = txt.trim().parse::<usize>().ok();
                }
            }
            "input-path-pattern" => {
                if let Some(txt) = child.text() {
                    input_path_pattern = Some(txt.trim().to_string());
                }
            }
            "answer-path-pattern" => {
                if let Some(txt) = child.text() {
                    answer_path_pattern = Some(txt.trim().to_string());
                }
            }
            "tests" => {
                for (idx, test_node) in child
                    .children()
                    .filter(|c| c.is_element() && c.tag_name().name() == "test")
                    .enumerate()
                {
                    standalone_test_cases.push(parse_test_case_node(&test_node, Some(idx + 1)));
                }
            }
            "test-group" => {
                let comment_description = child.attribute("comment").map(|s| s.to_string());
                let scoring_strategy = child.attribute("score").map(|s| s.to_string());
                let feedback_strategy = child.attribute("feedback").map(|s| s.to_string());
                let group_bonus_points = child
                    .attribute("group-bonus")
                    .and_then(|s| s.parse::<f64>().ok());
                let require_previous_group = child
                    .attribute("require-previous")
                    .map(|s| s == "true" || s == "1");

                let required_group_identifiers = child
                    .attribute("require-groups")
                    .map(|s| {
                        s.split_whitespace()
                            .map(|g| g.trim().to_string())
                            .filter(|g| !g.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();

                let mut group_tests = Vec::new();
                for (idx, test_node) in child
                    .children()
                    .filter(|c| c.is_element() && c.tag_name().name() == "test")
                    .enumerate()
                {
                    group_tests.push(parse_test_case_node(&test_node, Some(idx + 1)));
                }

                test_groups.push(Pcms2TestGroup {
                    group_identifier: None,
                    comment_description,
                    scoring_strategy,
                    feedback_strategy,
                    group_bonus_points,
                    require_previous_group,
                    required_group_identifiers,
                    test_cases: group_tests,
                });
            }
            "test" => {
                standalone_test_cases.push(parse_test_case_node(&child, None));
            }
            _ => {}
        }
    }

    Pcms2TestSet {
        testset_name,
        time_limit_milliseconds,
        memory_limit_bytes,
        test_count,
        input_file_name,
        output_file_name,
        input_path_pattern,
        answer_path_pattern,
        test_groups,
        standalone_test_cases,
    }
}

fn parse_test_case_node(
    node: &roxmltree::Node<'_, '_>,
    fallback_index: Option<usize>,
) -> Pcms2TestCase {
    let group_identifier = node.attribute("group").map(|s| s.to_string());
    let generation_method = node.attribute("method").map(|s| s.to_string());
    let is_sample_test = node
        .attribute("sample")
        .map(|s| s == "true" || s == "1")
        .unwrap_or(false);
    let generation_command = node.attribute("cmd").map(|s| s.to_string());
    let from_file_identifier = node.attribute("from-file").map(|s| s.to_string());
    let score_points = node.attribute("points").and_then(|s| s.parse::<f64>().ok());
    let comment_description = node.attribute("comment").map(|s| s.to_string());

    Pcms2TestCase {
        test_index_or_number: fallback_index,
        group_identifier,
        generation_method,
        is_sample_test,
        generation_command,
        from_file_identifier,
        score_points,
        comment_description,
    }
}

fn parse_time_limit_str(val: &str) -> Option<u64> {
    let trimmed = val.trim();
    if trimmed.ends_with('s') || trimmed.ends_with('S') {
        let num_str = &trimmed[..trimmed.len() - 1].trim();
        let seconds = num_str.parse::<f64>().ok()?;
        Some((seconds * 1000.0).round() as u64)
    } else {
        trimmed.parse::<u64>().ok()
    }
}

fn parse_memory_limit_str(val: &str) -> Option<u64> {
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

fn find_pcms2_xml_file(folder_path: &Path) -> Result<PathBuf, ParserError> {
    if folder_path.is_file() {
        return Ok(folder_path.to_path_buf());
    }

    let direct = folder_path.join("problem.xml");
    if direct.is_file() {
        return Ok(direct);
    }

    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("xml"))
                    .unwrap_or(false)
            {
                if let Ok(raw_bytes) = fs::read(&path) {
                    if let Ok(text) = decode_text(&raw_bytes) {
                        if text.contains("<problem") {
                            return Ok(path);
                        }
                    }
                }
            }
        }
    }

    Err(ParserError::FileNotFound(format!(
        "PCMS2 'problem.xml' was not found in directory '{}'",
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