use eframe::run_native;
use petgraph::prelude::{DiGraphMap, StableGraph};
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
mod app;

fn read_file(path: &str) -> String {
    let contents =
        fs::read_to_string(path).expect(format!("Cannot read input file: {}", path).as_str());
    contents
}

fn tokenize(contents: String) -> Vec<String> {
    contents
        .split_whitespace()
        .map(|token| token.to_string())
        .collect()
}

fn _is_pascal_case(token: &String) -> bool {
    let mut prev_is_upper: bool = false;

    for (idx, c) in token.chars().enumerate() {
        if (idx == 0) && (!c.is_uppercase()) {
            return false;
        }

        if c.is_uppercase() {
            if prev_is_upper {
                return false;
            }
            prev_is_upper = true;
        } else if c.is_lowercase() {
            prev_is_upper = false;
        }
    }
    true
}

fn get_pascal_case<'a>(tokens: &'a Vec<String>) -> Vec<&'a String> {
    let mut classes: Vec<&String> = vec![];
    let mut prev_token_class: bool = false;
    for token in tokens.iter() {
        if _is_pascal_case(token) == true && token.ends_with(':') && prev_token_class == true {
            classes.push(token)
        }

        if token == "class" {
            prev_token_class = true;
        } else {
            prev_token_class = false;
        }
    }
    classes
}

fn _is_child<'a>(class: &'a String) -> bool {
    class.contains(&['(', ')'])
}

fn get_child_classes<'a>(classes: Vec<&'a String>) -> Vec<&'a String> {
    let mut child_classes = vec![];

    for class in classes.iter() {
        if _is_child(class) == true {
            child_classes.push(*class);
        }
    }
    child_classes
}

fn get_parent_class<'a>(child_class: &'a String) -> String {
    let mut begin_parent_class: bool = false;
    let mut parent_class = String::new();
    for ch in child_class.chars() {
        if ch == ')' {
            begin_parent_class = false;
        }

        if begin_parent_class == true {
            parent_class.push(ch);
        }

        if ch == '(' {
            begin_parent_class = true;
        }
    }
    parent_class
}

fn _clean_child_class<'a>(child_class: &'a String) -> Option<String> {
    let mut tokens = child_class
        .split('(')
        .map(|token| token.to_string())
        .collect::<Vec<String>>();
    let mut token = tokens.drain(0..1);
    token.next()
}

fn build_edges<'a>(
    mut edges: Vec<(String, String)>,
    child_classes: Vec<&'a String>,
) -> Vec<(String, String)> {
    for class in child_classes.iter() {
        let parent_class = get_parent_class(class);
        let child_class = _clean_child_class(*class).unwrap();
        edges.push((child_class, parent_class));
    }
    edges
}

struct CommandLineConfig {
    file_path: Option<String>,
    module: Option<String>,
}

impl CommandLineConfig {
    fn new(args: &[String]) -> Self {
        if args.len() < 1 {
            panic!("not enough arguments");
        }
        if args[1].ends_with(".py") {
            Self {
                file_path: Some(args[1].clone()),
                module: None,
            }
        } else {
            Self {
                file_path: None,
                module: Some(args[1].clone()),
            }
        }
    }
}

fn parse_file<'a>(contents: String) -> Vec<(String, String)> {
    let tokens = tokenize(contents);

    let classes = get_pascal_case(&tokens);

    let child_classes = get_child_classes(classes);

    let edges = vec![];

    build_edges(edges, child_classes)
}

fn build_graph<'a>(edges: &'static [(String, String)]) -> petgraph::Graph<&'a str, i32> {
    let mut graph = DiGraphMap::new();
    for edge in edges.iter() {
        graph.add_edge(edge.1.as_str(), edge.0.as_str(), -1);
    }
    graph.into_graph()
}

struct ReadModule {
    files: Vec<String>,
}

impl ReadModule {
    fn new() -> Self {
        Self { files: vec![] }
    }

    fn read(&mut self, module_path: &Path) -> io::Result<()> {
        if module_path.is_dir() {
            for entry in fs::read_dir(module_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.read(&path)?;
                } else {
                    if entry.path().extension().is_some()
                        && entry.path().extension().unwrap() == "py"
                    {
                        self.files.push(read_file(entry.path().to_str().unwrap()));
                    }
                }
            }
        }
        Ok(())
    }
}

fn process_files(contents: Vec<String>) -> &'static [(String, String)] {
    let edges = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = contents
        .into_iter()
        .map(|content| {
            let edges = Arc::clone(&edges);
            thread::spawn(move || {
                let processed = parse_file(content.to_string());
                let mut res = edges.lock().unwrap();
                res.extend(processed);
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread failed");
    }

    let edges = edges.lock().unwrap();

    Box::leak(edges.to_vec().into_boxed_slice())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = CommandLineConfig::new(&args);

    let mut contents = vec![];

    if config.file_path.is_some() {
        contents.push(read_file(config.file_path.unwrap().as_str()))
    } else {
        let mut module_reader = ReadModule::new();
        let _ = module_reader.read(Path::new(config.module.unwrap().as_str()));
        contents = module_reader.files;
    };

    let edges: &'static [(String, String)] = process_files(contents);

    let graph = build_graph(edges);

    let stable_graph = StableGraph::from(graph);

    let mut native_options = eframe::NativeOptions::default();

    native_options.viewport.maximized = Some(true);

    run_native(
        "",
        native_options,
        Box::new(|cc| Ok(Box::new(app::OOPViewerApp::new(stable_graph, cc)))),
    )
    .unwrap();
}
