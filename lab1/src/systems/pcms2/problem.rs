use crate::core::common::{
    CommonProblemSummary, CommonTestCase, CommonTestGroup, DataStreamSource, ProblemResourceLimits,
};

/// Localized name of the problem in PCMS2 package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcms2ProblemName {
    pub language: String,
    pub problem_title: String,
}

/// Problem statement or tutorial document reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcms2DocumentReference {
    pub character_set: Option<String>,
    pub language: Option<String>,
    pub file_path: String,
    pub mime_type: Option<String>,
}

/// Executable file reference (source code and compiled binary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcms2Executable {
    pub source_path: Option<String>,
    pub source_type: Option<String>,
    pub binary_path: Option<String>,
    pub binary_type: Option<String>,
    pub executable_identifier: Option<String>,
}

/// Checker specification in PCMS2 package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcms2Checker {
    pub checker_name: Option<String>,
    pub checker_type: Option<String>,
    pub executable: Option<Pcms2Executable>,
}

/// Solution specification in PCMS2 package with tag (accepted, main, wrong-answer, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcms2Solution {
    pub verdict_tag: String,
    pub executable: Pcms2Executable,
}

/// Resource file (style, header, template) in PCMS2 package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcms2ResourceFile {
    pub file_path: String,
    pub file_type: Option<String>,
}

/// Single test case description in PCMS2 package.
#[derive(Debug, Clone, PartialEq)]
pub struct Pcms2TestCase {
    pub test_index_or_number: Option<usize>,
    pub group_identifier: Option<String>,
    pub generation_method: Option<String>,
    pub is_sample_test: bool,
    pub generation_command: Option<String>,
    pub from_file_identifier: Option<String>,
    pub score_points: Option<f64>,
    pub comment_description: Option<String>,
}

/// Group of tests in PCMS2 package.
#[derive(Debug, Clone, PartialEq)]
pub struct Pcms2TestGroup {
    pub group_identifier: Option<String>,
    pub comment_description: Option<String>,
    pub scoring_strategy: Option<String>,
    pub feedback_strategy: Option<String>,
    pub group_bonus_points: Option<f64>,
    pub require_previous_group: Option<bool>,
    pub required_group_identifiers: Vec<String>,
    pub test_cases: Vec<Pcms2TestCase>,
}

/// Testset specification in PCMS2 judging block.
#[derive(Debug, Clone, PartialEq)]
pub struct Pcms2TestSet {
    pub testset_name: String,
    pub time_limit_milliseconds: Option<u64>,
    pub memory_limit_bytes: Option<u64>,
    pub test_count: Option<usize>,
    pub input_file_name: Option<String>,
    pub output_file_name: Option<String>,
    pub input_path_pattern: Option<String>,
    pub answer_path_pattern: Option<String>,
    pub test_groups: Vec<Pcms2TestGroup>,
    pub standalone_test_cases: Vec<Pcms2TestCase>,
}

/// Complete representation of a PCMS2 problem package.
#[derive(Debug, Clone, PartialEq)]
pub struct Pcms2Problem {
    pub specification_version: Option<String>,
    pub problem_global_identifier: Option<String>,
    pub short_name: Option<String>,
    pub revision_number: Option<String>,
    pub polygon_url: Option<String>,
    pub localized_names: Vec<Pcms2ProblemName>,
    pub statement_documents: Vec<Pcms2DocumentReference>,
    pub tutorial_documents: Vec<Pcms2DocumentReference>,
    pub cpu_name: Option<String>,
    pub cpu_speed_mhz: Option<u32>,
    pub default_input_file_name: Option<String>,
    pub default_output_file_name: Option<String>,
    pub judging_script_type: Option<String>,
    pub test_sets: Vec<Pcms2TestSet>,
    pub resource_files: Vec<Pcms2ResourceFile>,
    pub executable_tools: Vec<Pcms2Executable>,
    pub checker: Option<Pcms2Checker>,
    pub interactor: Option<Pcms2Executable>,
    pub validators: Vec<Pcms2Executable>,
    pub solutions: Vec<Pcms2Solution>,
    pub general_documents: Vec<Pcms2DocumentReference>,
    pub tags: Vec<String>,
    pub common_summary: CommonProblemSummary,
}

impl Pcms2Problem {
    pub fn build_common_summary(&self) -> CommonProblemSummary {
        let first_set = self.test_sets.first();
        let time_limit_milliseconds = first_set.and_then(|s| s.time_limit_milliseconds);
        let memory_limit_bytes = first_set.and_then(|s| s.memory_limit_bytes);

        let input_file = first_set
            .and_then(|s| s.input_file_name.clone())
            .or_else(|| self.default_input_file_name.clone())
            .filter(|s| !s.trim().is_empty());

        let output_file = first_set
            .and_then(|s| s.output_file_name.clone())
            .or_else(|| self.default_output_file_name.clone())
            .filter(|s| !s.trim().is_empty());

        let input_stream = match input_file {
            Some(f) => DataStreamSource::SpecificFile(f),
            None => DataStreamSource::StandardInputOutput,
        };

        let output_stream = match output_file {
            Some(f) => DataStreamSource::SpecificFile(f),
            None => DataStreamSource::StandardInputOutput,
        };

        let mut total_test_cases_count = 0usize;
        let mut sample_tests_count = 0usize;
        let mut total_test_groups_count = 0usize;
        let mut common_test_groups = Vec::new();
        let mut sample_tests = Vec::new();

        for set in &self.test_sets {
            total_test_groups_count += set.test_groups.len();
            for group in &set.test_groups {
                let group_id = group
                    .group_identifier
                    .clone()
                    .or_else(|| group.comment_description.clone())
                    .unwrap_or_else(|| format!("group_{}", common_test_groups.len() + 1));

                common_test_groups.push(CommonTestGroup {
                    group_identifier: group_id,
                    group_description: group.comment_description.clone(),
                    total_score_points: group.group_bonus_points,
                    single_test_points: None,
                    prerequisite_group_identifiers: group.required_group_identifiers.clone(),
                    included_test_count: group.test_cases.len(),
                });

                for test in &group.test_cases {
                    total_test_cases_count += 1;
                    if test.is_sample_test {
                        sample_tests_count += 1;
                        sample_tests.push(CommonTestCase {
                            test_index_or_number: test.test_index_or_number,
                            input_file_or_description: test
                                .generation_command
                                .clone()
                                .unwrap_or_else(|| "sample input".to_string()),
                            answer_file_or_description: None,
                            assigned_group_identifier: test.group_identifier.clone(),
                            score_points: test.score_points,
                            is_sample_test: true,
                        });
                    }
                }
            }

            for test in &set.standalone_test_cases {
                total_test_cases_count += 1;
                if test.is_sample_test {
                    sample_tests_count += 1;
                    sample_tests.push(CommonTestCase {
                        test_index_or_number: test.test_index_or_number,
                        input_file_or_description: test
                            .generation_command
                            .clone()
                            .unwrap_or_else(|| "sample input".to_string()),
                        answer_file_or_description: None,
                        assigned_group_identifier: test.group_identifier.clone(),
                        score_points: test.score_points,
                        is_sample_test: true,
                    });
                }
            }

            if let Some(count) = set.test_count {
                if count > total_test_cases_count {
                    total_test_cases_count = count;
                }
            }
        }

        let primary_title = self
            .localized_names
            .first()
            .map(|n| n.problem_title.clone())
            .or_else(|| self.short_name.clone())
            .or_else(|| self.problem_global_identifier.clone());

        let checker_description = self
            .checker
            .as_ref()
            .and_then(|c| {
                c.checker_name
                    .clone()
                    .or_else(|| c.executable.as_ref().and_then(|e| e.binary_path.clone()))
            });

        let interactor_description = self
            .interactor
            .as_ref()
            .and_then(|i| i.binary_path.clone().or_else(|| i.source_path.clone()));

        CommonProblemSummary {
            problem_title: primary_title,
            problem_identifier: self
                .short_name
                .clone()
                .or_else(|| self.problem_global_identifier.clone()),
            resource_limits: ProblemResourceLimits {
                time_limit_milliseconds,
                memory_limit_bytes,
                write_limit_bytes: None,
            },
            input_stream,
            output_stream,
            checker_description,
            interactor_description,
            total_test_cases_count,
            total_test_groups_count,
            sample_tests_count,
            test_groups: common_test_groups,
            sample_tests,
        }
    }
}