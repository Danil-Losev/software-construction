use lab1::core::registry;
use lab1::systems::cats;
use lab1::systems::ejudge;
use lab1::systems::krsu;
use lab1::systems::pcms2;

fn main() {

    let problem_type = std::env::args().nth(1).expect("Please provide a problem type as the first argument.");
    let problem_folder_path = std::env::args().nth(2).expect("Please provide a problem folder path as the second argument.");

    let problem_type = problem_type.to_lowercase();
    let path = std::path::Path::new(&problem_folder_path);
    if !path.exists() || !path.is_dir() {
        eprintln!("The provided problem folder path '{}' does not exist or is not a directory.", problem_folder_path);
        std::process::exit(1);
    }

    registry::register_system::<cats::CatsSystem>();
    registry::register_system::<ejudge::EjudgeSystem>();

    
    registry::register_system::<krsu::KrsuSystem>();
    registry::register_system::<pcms2::Pcms2System>();

    let map = registry::get_registry().read().expect("Registry lock poisoned");
    if let Some(pipeline) = map.get(problem_type.as_str()) {
        pipeline(path);
    } else {
        eprintln!("System '{}' not found in registry.", problem_type);
        std::process::exit(1);
    }
}
