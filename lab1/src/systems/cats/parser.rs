use std::fs;
use std::path::{Path, PathBuf};

use crate::core::parser::{Parser, ParserError};
use super::problem::{
    CatsAttachment, CatsChecker, CatsCheckerStyle, CatsDevelopmentEnvironment, CatsGenerator,
    CatsImport, CatsInteractor, CatsPicture, CatsProblem, CatsProblemStatement, CatsSample,
    CatsSolution, CatsTest, CatsTestInputDefinition, CatsTestOutputDefinition, CatsTestRange,
    CatsTestset, CatsValidator,
};

pub struct CatsParser;

impl Parser for CatsParser {
    type Output = CatsProblem;

    fn parse(&self, folder_path: &Path) -> Result<Self::Output, ParserError> {
        let xml_path = find_cats_xml_file(folder_path)?;
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

        let cats_element = document.root_element();
        if cats_element.tag_name().name() != "CATS" {
            return Err(ParserError::InvalidSyntax(format!(
                "Expected root element <CATS>, found <{}> in '{}'",
                cats_element.tag_name().name(),
                xml_path.display()
            )));
        }

        let cats_specification_version = cats_element
            .attribute("version")
            .unwrap_or("1.4")
            .to_string();

        let problem_element = cats_element
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "Problem")
            .ok_or_else(|| {
                ParserError::InvalidSyntax(format!(
                    "Missing <Problem> tag inside <CATS> in '{}'",
                    xml_path.display()
                ))
            })?;

        let problem_title = problem_element
            .attribute("title")
            .unwrap_or("Untitled Problem")
            .to_string();

        let supported_natural_languages = problem_element
            .attribute("lang")
            .map(|l| {
                l.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let time_limit_seconds = problem_element
            .attribute("tlimit")
            .and_then(|t| t.parse::<f64>().ok());

        let memory_limit_bytes = problem_element
            .attribute("mlimit")
            .and_then(parse_memory_size_string);

        let write_limit_bytes = problem_element
            .attribute("wlimit")
            .and_then(parse_memory_size_string);

        let author_name = problem_element
            .attribute("author")
            .map(|a| a.trim().to_string());
        let input_file_name = problem_element
            .attribute("inputFile")
            .map(|f| f.trim().to_string());
        let output_file_name = problem_element
            .attribute("outputFile")
            .map(|f| f.trim().to_string());
        let author_difficulty_rating = problem_element
            .attribute("difficulty")
            .and_then(|d| d.parse::<u32>().ok());
        let standard_checker_identifier = problem_element
            .attribute("stdChecker")
            .map(|c| c.trim().to_string());
        let maximum_score_points = problem_element
            .attribute("maxPoints")
            .and_then(|p| p.parse::<f64>().ok());
        let save_input_prefix_limit = problem_element
            .attribute("saveInputPrefix")
            .map(|p| p.to_string());
        let save_output_prefix_limit = problem_element
            .attribute("saveOutputPrefix")
            .map(|p| p.to_string());
        let save_answer_prefix_limit = problem_element
            .attribute("saveAnswerPrefix")
            .map(|p| p.to_string());

        let mut keyword_codes = Vec::new();
        let mut problem_statement = None;
        let mut problem_constraints_description = None;
        let mut input_format_description = None;
        let mut output_format_description = None;
        let mut solution_explanation_description = None;
        let mut json_metadata = None;
        let mut pictures = Vec::new();
        let mut attachments = Vec::new();
        let mut reference_solutions = Vec::new();
        let mut custom_checkers = Vec::new();
        let mut test_generators = Vec::new();
        let mut test_validators = Vec::new();
        let mut interactor = None;
        let mut run_execution_method = None;
        let mut sample_demonstrations = Vec::new();
        let mut test_definitions = Vec::new();
        let mut test_ranges = Vec::new();
        let mut test_sets = Vec::new();
        let mut imported_dependencies = Vec::new();

        for child in problem_element.children() {
            if !child.is_element() {
                continue;
            }

            match child.tag_name().name() {
                "Keyword" => {
                    if let Some(code) = child.attribute("code") {
                        keyword_codes.push(code.trim().to_string());
                    }
                }
                "ProblemStatement" => {
                    let attachment_identifier = child.attribute("attachment").map(|s| s.to_string());
                    let external_url = child.attribute("url").map(|s| s.to_string());
                    let content_stml = extract_inner_text_or_stml(&child);
                    problem_statement = Some(CatsProblemStatement {
                        content_stml,
                        attachment_identifier,
                        external_url,
                    });
                }
                "ProblemConstraints" => {
                    problem_constraints_description = Some(extract_inner_text_or_stml(&child));
                }
                "InputFormat" => {
                    input_format_description = Some(extract_inner_text_or_stml(&child));
                }
                "OutputFormat" => {
                    output_format_description = Some(extract_inner_text_or_stml(&child));
                }
                "Explanation" => {
                    solution_explanation_description = Some(extract_inner_text_or_stml(&child));
                }
                "JsonData" => {
                    json_metadata = Some(extract_inner_text_or_stml(&child));
                }
                "Picture" => {
                    let name = child.attribute("name").unwrap_or_default().to_string();
                    let source_path = child.attribute("src").unwrap_or_default().to_string();
                    pictures.push(CatsPicture { name, source_path });
                }
                "Attachment" => {
                    let name = child.attribute("name").unwrap_or_default().to_string();
                    let source_path = child.attribute("src").unwrap_or_default().to_string();
                    attachments.push(CatsAttachment { name, source_path });
                }
                "Solution" => {
                    let solution_name = child.attribute("name").unwrap_or_default().to_string();
                    let source_file_path = child.attribute("src").unwrap_or_default().to_string();
                    let development_environment =
                        CatsDevelopmentEnvironment::from_code(child.attribute("de_code"));
                    let global_export_identifier = child.attribute("export").map(|s| s.to_string());
                    let time_limit_seconds = child
                        .attribute("timeLimit")
                        .and_then(|t| t.parse::<f64>().ok());
                    let memory_limit_bytes = child
                        .attribute("memoryLimit")
                        .and_then(parse_memory_size_string);
                    let write_limit_bytes = child
                        .attribute("writeLimit")
                        .and_then(parse_memory_size_string);

                    reference_solutions.push(CatsSolution {
                        solution_name,
                        source_file_path,
                        development_environment,
                        global_export_identifier,
                        time_limit_seconds,
                        memory_limit_bytes,
                        write_limit_bytes,
                    });
                }
                "Checker" => {
                    let checker_name = child.attribute("name").unwrap_or_default().to_string();
                    let source_file_path = child.attribute("src").unwrap_or_default().to_string();
                    let development_environment =
                        CatsDevelopmentEnvironment::from_code(child.attribute("de_code"));
                    let checker_parameter_style =
                        CatsCheckerStyle::from_style(child.attribute("style"));
                    let global_export_identifier = child.attribute("export").map(|s| s.to_string());
                    let time_limit_seconds = child
                        .attribute("timeLimit")
                        .and_then(|t| t.parse::<f64>().ok());
                    let memory_limit_bytes = child
                        .attribute("memoryLimit")
                        .and_then(parse_memory_size_string);
                    let write_limit_bytes = child
                        .attribute("writeLimit")
                        .and_then(parse_memory_size_string);

                    custom_checkers.push(CatsChecker {
                        checker_name,
                        source_file_path,
                        development_environment,
                        checker_parameter_style,
                        global_export_identifier,
                        time_limit_seconds,
                        memory_limit_bytes,
                        write_limit_bytes,
                    });
                }
                "Generator" => {
                    let generator_name = child.attribute("name").unwrap_or_default().to_string();
                    let source_file_path = child.attribute("src").unwrap_or_default().to_string();
                    let output_file_pattern = child.attribute("outputFile").map(|s| s.to_string());
                    let development_environment =
                        CatsDevelopmentEnvironment::from_code(child.attribute("de_code"));
                    let global_export_identifier = child.attribute("export").map(|s| s.to_string());
                    let time_limit_seconds = child
                        .attribute("timeLimit")
                        .and_then(|t| t.parse::<f64>().ok());
                    let memory_limit_bytes = child
                        .attribute("memoryLimit")
                        .and_then(parse_memory_size_string);
                    let write_limit_bytes = child
                        .attribute("writeLimit")
                        .and_then(parse_memory_size_string);

                    test_generators.push(CatsGenerator {
                        generator_name,
                        source_file_path,
                        output_file_pattern,
                        development_environment,
                        global_export_identifier,
                        time_limit_seconds,
                        memory_limit_bytes,
                        write_limit_bytes,
                    });
                }
                "Validator" => {
                    let validator_name = child.attribute("name").unwrap_or_default().to_string();
                    let source_file_path = child.attribute("src").unwrap_or_default().to_string();
                    let development_environment =
                        CatsDevelopmentEnvironment::from_code(child.attribute("de_code"));
                    let global_export_identifier = child.attribute("export").map(|s| s.to_string());
                    let time_limit_seconds = child
                        .attribute("timeLimit")
                        .and_then(|t| t.parse::<f64>().ok());
                    let memory_limit_bytes = child
                        .attribute("memoryLimit")
                        .and_then(parse_memory_size_string);
                    let write_limit_bytes = child
                        .attribute("writeLimit")
                        .and_then(parse_memory_size_string);

                    test_validators.push(CatsValidator {
                        validator_name,
                        source_file_path,
                        development_environment,
                        global_export_identifier,
                        time_limit_seconds,
                        memory_limit_bytes,
                        write_limit_bytes,
                    });
                }
                "Interactor" => {
                    let interactor_name = child.attribute("name").unwrap_or_default().to_string();
                    let source_file_path = child.attribute("src").unwrap_or_default().to_string();
                    let development_environment =
                        CatsDevelopmentEnvironment::from_code(child.attribute("de_code"));
                    let global_export_identifier = child.attribute("export").map(|s| s.to_string());
                    let time_limit_seconds = child
                        .attribute("timeLimit")
                        .and_then(|t| t.parse::<f64>().ok());
                    let memory_limit_bytes = child
                        .attribute("memoryLimit")
                        .and_then(parse_memory_size_string);
                    let write_limit_bytes = child
                        .attribute("writeLimit")
                        .and_then(parse_memory_size_string);

                    interactor = Some(CatsInteractor {
                        interactor_name,
                        source_file_path,
                        development_environment,
                        global_export_identifier,
                        time_limit_seconds,
                        memory_limit_bytes,
                        write_limit_bytes,
                    });
                }
                "Run" => {
                    run_execution_method = child.attribute("method").map(|s| s.to_string());
                }
                "Import" => {
                    let global_unique_identifier =
                        child.attribute("guid").unwrap_or_default().to_string();
                    let imported_object_type = child.attribute("type").map(|s| s.to_string());
                    let local_alias_name = child.attribute("name").map(|s| s.to_string());

                    imported_dependencies.push(CatsImport {
                        global_unique_identifier,
                        imported_object_type,
                        local_alias_name,
                    });
                }
                "Sample" => {
                    let rank_specification = child.attribute("rank").unwrap_or("1").to_string();
                    let mut input_file_path = None;
                    let mut input_inline_text = None;
                    let mut output_file_path = None;
                    let mut output_inline_text = None;

                    for sample_child in child.children().filter(|c| c.is_element()) {
                        match sample_child.tag_name().name() {
                            "SampleIn" => {
                                input_file_path = sample_child.attribute("src").map(|s| s.to_string());
                                input_inline_text = sample_child.text().map(|s| s.to_string());
                            }
                            "SampleOut" => {
                                output_file_path = sample_child.attribute("src").map(|s| s.to_string());
                                output_inline_text = sample_child.text().map(|s| s.to_string());
                            }
                            _ => {}
                        }
                    }

                    sample_demonstrations.push(CatsSample {
                        rank_specification,
                        input_file_path,
                        input_inline_text,
                        output_file_path,
                        output_inline_text,
                    });
                }
                "Test" => {
                    let rank_specification = child.attribute("rank").unwrap_or("1").to_string();
                    let score_points = child.attribute("points").and_then(|p| p.parse::<f64>().ok());
                    let mut input_definition = None;
                    let mut output_definition = None;

                    for sub in child.children().filter(|c| c.is_element()) {
                        match sub.tag_name().name() {
                            "In" => {
                                input_definition = Some(parse_cats_input_node(&sub));
                            }
                            "Out" => {
                                output_definition = Some(parse_cats_output_node(&sub));
                            }
                            _ => {}
                        }
                    }

                    test_definitions.push(CatsTest {
                        rank_specification,
                        score_points,
                        input_definition,
                        output_definition,
                    });
                }
                "TestRange" => {
                    let range_start_number = child
                        .attribute("from")
                        .and_then(|f| f.parse::<u32>().ok())
                        .unwrap_or(1);
                    let range_end_number = child
                        .attribute("to")
                        .and_then(|t| t.parse::<u32>().ok())
                        .unwrap_or(1);
                    let points_per_test = child
                        .attribute("points")
                        .and_then(|p| p.parse::<f64>().ok());

                    let mut input_definition = None;
                    let mut output_definition = None;

                    for sub in child.children().filter(|c| c.is_element()) {
                        match sub.tag_name().name() {
                            "In" => {
                                input_definition = Some(parse_cats_input_node(&sub));
                            }
                            "Out" => {
                                output_definition = Some(parse_cats_output_node(&sub));
                            }
                            _ => {}
                        }
                    }

                    test_ranges.push(CatsTestRange {
                        range_start_number,
                        range_end_number,
                        points_per_test,
                        input_definition,
                        output_definition,
                    });
                }
                "Testset" => {
                    let testset_name = child.attribute("name").unwrap_or_default().to_string();
                    let test_rank_specification =
                        child.attribute("tests").unwrap_or_default().to_string();
                    let total_score_points = child
                        .attribute("points")
                        .and_then(|p| p.parse::<f64>().ok());
                    let hide_details = child
                        .attribute("hideDetails")
                        .map(|h| h == "1" || h.eq_ignore_ascii_case("true"))
                        .unwrap_or(false);
                    let comment_description = child.attribute("comment").map(|s| s.to_string());
                    let depends_on_tests_specification =
                        child.attribute("depends_on").map(|s| s.to_string());

                    test_sets.push(CatsTestset {
                        testset_name,
                        test_rank_specification,
                        total_score_points,
                        hide_details,
                        comment_description,
                        depends_on_tests_specification,
                    });
                }
                _ => {}
            }
        }

        let mut problem = CatsProblem {
            cats_specification_version,
            problem_title,
            supported_natural_languages,
            time_limit_seconds,
            memory_limit_bytes,
            write_limit_bytes,
            author_name,
            input_file_name,
            output_file_name,
            author_difficulty_rating,
            standard_checker_identifier,
            maximum_score_points,
            save_input_prefix_limit,
            save_output_prefix_limit,
            save_answer_prefix_limit,
            keyword_codes,
            problem_statement,
            problem_constraints_description,
            input_format_description,
            output_format_description,
            solution_explanation_description,
            json_metadata,
            pictures,
            attachments,
            reference_solutions,
            custom_checkers,
            test_generators,
            test_validators,
            interactor,
            run_execution_method,
            sample_demonstrations,
            test_definitions,
            test_ranges,
            test_sets,
            imported_dependencies,
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

fn parse_cats_input_node(node: &roxmltree::Node<'_, '_>) -> CatsTestInputDefinition {
    CatsTestInputDefinition {
        source_file_path: node.attribute("src").map(|s| s.to_string()),
        generator_reference_name: node.attribute("use").map(|s| s.to_string()),
        generator_parameters: node.attribute("param").map(|s| s.to_string()),
        use_group_generation: node.attribute("genAll").map(|g| g == "1").unwrap_or(false),
        validator_reference_name: node.attribute("validate").map(|s| s.to_string()),
        file_checksum_hash: node.attribute("hash").map(|s| s.to_string()),
        inline_content: node.text().map(|s| s.to_string()),
    }
}

fn parse_cats_output_node(node: &roxmltree::Node<'_, '_>) -> CatsTestOutputDefinition {
    CatsTestOutputDefinition {
        source_file_path: node.attribute("src").map(|s| s.to_string()),
        solution_reference_name: node.attribute("use").map(|s| s.to_string()),
        inline_content: node.text().map(|s| s.to_string()),
    }
}

fn extract_inner_text_or_stml(node: &roxmltree::Node<'_, '_>) -> String {
    let mut text_parts = Vec::new();
    collect_text_recursive(node, &mut text_parts);
    text_parts.join(" ").trim().to_string()
}

fn collect_text_recursive(node: &roxmltree::Node<'_, '_>, buffer: &mut Vec<String>) {
    if let Some(t) = node.text() {
        let trimmed = t.trim();
        if !trimmed.is_empty() {
            buffer.push(trimmed.to_string());
        }
    }
    for child in node.children() {
        collect_text_recursive(&child, buffer);
    }
}

fn parse_memory_size_string(val: &str) -> Option<u64> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return None;
    }

    let last_char = trimmed.chars().last()?;
    if last_char.is_ascii_digit() {
        // In CATS specification, if suffix is omitted, megabytes are assumed
        let mb = trimmed.parse::<u64>().ok()?;
        return Some(mb * 1024 * 1024);
    }

    let unit = last_char.to_ascii_uppercase();
    let num_str = &trimmed[..trimmed.len() - 1].trim();
    let num = num_str.parse::<u64>().ok()?;

    match unit {
        'B' => Some(num),
        'K' => Some(num * 1024),
        'M' => Some(num * 1024 * 1024),
        'G' => Some(num * 1024 * 1024 * 1024),
        _ => Some(num * 1024 * 1024),
    }
}

fn find_cats_xml_file(folder_path: &Path) -> Result<PathBuf, ParserError> {
    if folder_path.is_file() {
        return Ok(folder_path.to_path_buf());
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
                        if text.contains("<CATS") {
                            return Ok(path);
                        }
                    }
                }
            }
        }
    }

    Err(ParserError::FileNotFound(format!(
        "CATS XML specification file was not found in directory '{}'",
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