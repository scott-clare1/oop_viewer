use eframe::{App, CreationContext};
use egui::Context;
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, GraphView, SettingsInteraction, SettingsNavigation,
    SettingsStyle,
};
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::prelude::StableGraph;

pub struct BasicApp<'a> {
    g: egui_graphs::Graph<&'a str, i32>,
}

impl<'a> BasicApp<'a> {
    pub fn new(graph: StableGraph<&'a str, i32>, _: &CreationContext<'_>) -> Self {
        let mut g = egui_graphs::Graph::from(&graph);
        for (idx, class) in graph.node_weights().enumerate() {
            g.node_mut(NodeIndex::new(idx))
                .unwrap()
                .set_label(class.to_string());
            let edge = g.edge_mut(EdgeIndex::new(idx));
            if edge.is_some() {
                edge.unwrap().set_label("".to_string());
            }
        }
        Self { g }
    }
}

impl<'a> App for BasicApp<'a> {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        let widget =
            &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(&mut self.g)
                .with_interactions(
                    &SettingsInteraction::default()
                        .with_dragging_enabled(true)
                        .with_node_selection_enabled(true)
                        .with_edge_selection_enabled(true),
                )
                .with_navigations(
                    &SettingsNavigation::default()
                        .with_fit_to_screen_enabled(false)
                        .with_zoom_and_pan_enabled(true),
                )
                .with_styles(&SettingsStyle::new().with_labels_always(true));
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(widget);
        });
    }
}
