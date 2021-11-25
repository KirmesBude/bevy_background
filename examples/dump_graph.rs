use bevy::{prelude::*, render::render_graph::RenderGraph};
use bevy_background::BackgroundPlugin;

fn main() {
    App::build()
        //.insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(BackgroundPlugin)
        .add_startup_system(print_render_graph.system())
        .run();
}

fn print_render_graph(render_graph: Res<RenderGraph>) {
    let dot = bevy_mod_debugdump::render_graph::render_graph_dot(&render_graph);
    std::fs::write("render-graph.dot", dot).expect("Failed to write render-graph.dot");
    println!("Render graph written to render-graph.dot");
}
