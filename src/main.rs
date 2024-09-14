use eframe::run_native;
use petgraph::prelude::{DiGraphMap, StableGraph};
use std::collections::{HashMap, HashSet, VecDeque};
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

fn separate_child_and_parent_class<'a>(class: &'a String) -> (String, Vec<String>) {
    let mut begin_parent_class: bool = false;
    let mut parent_class = String::new();
    let mut child_class = String::new();
    for ch in class.chars() {
        if ch == ')' {
            break;
        }

        if ch == '(' {
            begin_parent_class = true;
            continue;
        }

        if begin_parent_class == true {
            parent_class.push(ch);
        } else {
            child_class.push(ch);
        }
    }
    (
        child_class,
        parent_class
            .split(", ")
            .map(|token| token.to_string())
            .collect(),
    )
}

fn build_edges(child_classes: Vec<String>) -> Vec<(String, String)> {
    let mut edges = vec![];
    for class in child_classes.iter() {
        let (child_class, parent_classes) = separate_child_and_parent_class(class);
        for parent_class in parent_classes.iter() {
            edges.push((child_class.to_string(), parent_class.to_string()));
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

trait Graph {
    fn build_graph(self, edges: &Vec<(String, String)>) -> HashMap<String, Vec<String>>;

    fn bfs(&self, class: &String) -> HashSet<String>;
}

impl Graph for HashMap<String, Vec<String>> {
    fn build_graph(mut self, edges: &Vec<(String, String)>) -> HashMap<String, Vec<String>> {
        for (child, parent) in edges.iter() {
            if !parent.is_empty() {
                self.entry(parent.clone()).or_default().push(child.clone());
            }
        }
        self
    }

    fn bfs(&self, class: &String) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(class.to_string());

        while let Some(current_class) = queue.pop_front() {
            if !visited.contains(&current_class) {
                visited.insert(current_class.clone());
                if let Some(children) = self.get(&current_class) {
                    for child in children {
                        if !visited.contains(child) {
                            queue.push_back(child.clone());
                        }
                    }
                }
            }
        }
        visited
    }
}

fn filter_edges_by_class(
    edges: Vec<(String, String)>,
    class: Option<String>,
) -> Vec<(String, String)> {
    if class.is_some() {
        let class = class.unwrap();

        let visited = HashMap::new().build_graph(&edges).bfs(&class);

        edges
            .iter()
            .filter(|(child, parent)| visited.contains(child) || visited.contains(parent))
            .cloned()
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

trait AddEdges<'a> {
    fn add_edges(self, edges: &'static [(String, String)]) -> DiGraphMap<&'a str, i32>;
}

impl<'a> AddEdges<'a> for DiGraphMap<&'a str, i32> {
    fn add_edges(mut self, edges: &'static [(String, String)]) -> DiGraphMap<&'a str, i32> {
        for edge in edges.iter() {
            self.add_edge(edge.1.as_str(), edge.0.as_str(), -1);
        }
        self
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = CommandLineConfig::new(&args);

    let contents = extract_file_contents(config.file_path, config.module);

    let edges: &'static [(String, String)] = process_files(contents, config.class);

    let graph = DiGraphMap::new().add_edges(edges);

    let mut native_options = eframe::NativeOptions::default();

    native_options.viewport.maximized = Some(true);

    run_native(
        "",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(app::OOPViewerApp::new(
                StableGraph::from(graph.into_graph()),
                cc,
            )))
        }),
    )
    .unwrap();
}

#[cfg(test)]
mod tests;
