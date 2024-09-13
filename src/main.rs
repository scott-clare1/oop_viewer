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
    let content =
        fs::read_to_string(path).expect(format!("Cannot read input file: {}", path).as_str());
    content
}

fn tokenize(content: String) -> Vec<String> {
    content
        .split_whitespace()
        .map(|token| token.to_string())
        .collect()
}

fn is_pascal_case(token: &String) -> bool {
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

fn get_pascal_case<'a>(tokens: &'a Vec<String>) -> Vec<Vec<&String>> {
    let mut classes: Vec<Vec<&String>> = vec![];
    let mut prev_token_class: bool = false;
    let mut tokens_iter = tokens.iter();

    let mut token = tokens_iter.next();

    while token.is_some() {
        if is_pascal_case(token.unwrap()) == true && prev_token_class == true {
            if token.unwrap().ends_with(':') {
                classes.push(vec![token.unwrap()])
            } else if token.unwrap().ends_with(',') {
                let mut multi_inheritance_token = vec![];
                while !token.unwrap().ends_with(':') {
                    multi_inheritance_token.push(token.unwrap());
                    token = tokens_iter.next();
                }
                multi_inheritance_token.push(token.unwrap());
                classes.push(multi_inheritance_token)
            }
        }

        if token.unwrap() == "class" {
            prev_token_class = true;
        } else {
            prev_token_class = false;
        }

        token = tokens_iter.next();
    }

    classes
}

fn is_sinlge_inheritance_child<'a>(class: &'a String) -> bool {
    class.contains(&['(', ')'])
}

fn get_child_classes<'a>(classes: Vec<Vec<&String>>) -> Vec<String> {
    let mut child_classes = vec![];

    for class in classes.iter() {
        if is_sinlge_inheritance_child(class[0]) == true {
            child_classes.push(
                class
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
            );
        }
    }
    child_classes
}

fn get_parent_class<'a>(child_class: &'a String) -> Vec<String> {
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
        .split(", ")
        .map(|token| token.to_string())
        .collect()
}

fn clean_child_class<'a>(child_class: &'a String) -> Option<String> {
    let mut tokens = child_class
        .split('(')
        .map(|token| token.to_string())
        .collect::<Vec<String>>();
    let mut token = tokens.drain(0..1);
    token.next()
}

fn build_edges(child_classes: Vec<String>) -> Vec<(String, String)> {
    let mut edges = vec![];
    for class in child_classes.iter() {
        let parent_class = get_parent_class(class);
        let child_class = clean_child_class(class).unwrap();
        for parent in parent_class.iter() {
            edges.push((child_class.to_string(), parent.to_string()));
        }
    }
    edges
}

fn parse_file<'a>(contents: String) -> Vec<(String, String)> {
    let tokens = tokenize(contents);

    let classes = get_pascal_case(&tokens);

    let child_classes = get_child_classes(classes);

    build_edges(child_classes)
}

fn build_graph<'a>(edges: &'static [(String, String)]) -> StableGraph<&'a str, i32> {
    let mut graph = DiGraphMap::new();
    for edge in edges.iter() {
        graph.add_edge(edge.1.as_str(), edge.0.as_str(), -1);
    }
    StableGraph::from(graph.into_graph())
}

fn process_files(contents: Vec<String>, class: Option<String>) -> &'static [(String, String)] {
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

    Box::leak(filter_edges_by_class(edges.to_vec(), class).into_boxed_slice())
}

fn edge_contains_class((child, parent): (String, String), class: String) -> bool {
    if child == class || parent == class {
        true
    } else {
        false
    }
}

fn filter_edges_by_class(
    edges: Vec<(String, String)>,
    class: Option<String>,
) -> Vec<(String, String)> {
    if class.is_some() {
        let class = class.unwrap();
        edges
            .iter()
            .filter(|(child, parent)| {
                edge_contains_class((child.to_string(), parent.to_string()), class.clone())
            })
            .map(|(child, parent)| (child.to_string(), parent.to_string()))
            .collect()
    } else {
        edges
    }
}

fn extract_file_contents(file_path: Option<String>, module: Option<String>) -> Vec<String> {
    let mut contents = vec![];

    if file_path.is_some() {
        contents.push(read_file(file_path.unwrap().as_str()))
    } else {
        let mut module_reader = ReadModule::new();
        let _ = module_reader.read(Path::new(module.unwrap().as_str()));
        contents = module_reader.files;
    };
    contents
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

struct CommandLineConfig {
    file_path: Option<String>,
    module: Option<String>,
    class: Option<String>,
}

impl CommandLineConfig {
    fn new(args: &[String]) -> Self {
        let class = match args.len() {
            3 => Some(args[2].clone()),
            2 => None,
            _ => panic!(
                "Incorrect number of arguments. The tool should take either one or two positional arguments."
            ),
        };

        let (file_path, module) = match args[1].ends_with(".py") {
            true => (Some(args[1].clone()), None),
            false => (None, Some(args[1].clone())),
        };

        Self {
            file_path,
            module,
            class,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = CommandLineConfig::new(&args);

    let contents = extract_file_contents(config.file_path, config.module);

    let edges: &'static [(String, String)] = process_files(contents, config.class);

    let graph = build_graph(edges);

    let mut native_options = eframe::NativeOptions::default();

    native_options.viewport.maximized = Some(true);

    run_native(
        "",
        native_options,
        Box::new(|cc| Ok(Box::new(app::OOPViewerApp::new(graph, cc)))),
    )
    .unwrap();
}

#[cfg(test)]
mod tests;
