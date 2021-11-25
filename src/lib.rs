use bevy::{
    prelude::*,
    render::{
        camera::{ActiveCameras, Camera},
        pass::{
            LoadOp, Operations, PassDescriptor, RenderPassDepthStencilAttachmentDescriptor,
            TextureAttachment,
        },
        render_graph::{
            base::{self, MainPass},
            CameraNode, PassNode, RenderGraph, WindowSwapChainNode, WindowTextureNode,
        },
    },
};

/* Resource */
pub struct Background(pub Handle<ColorMaterial>);

/* Initialize the resource that determines the background to pink color */
impl FromWorld for Background {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();

        Background(materials.add(ColorMaterial::color(Color::PINK)))
    }
}

/* Plugin */
pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Background>()
            .add_startup_system(setup.system());
    }
}

fn setup(
    mut commands: Commands,
    mut render_graph: ResMut<RenderGraph>,
    mut active_cameras: ResMut<ActiveCameras>,
    background: Res<Background>,
    msaa: Res<Msaa>,
) {
    /* Add the background node */
    add_background_graph(&mut render_graph, &msaa);

    /* Spawn a spritebundle with our background material handle
       Also remove MainPass and add BackgroundPass instead
    */
    commands
        .spawn_bundle(SpriteBundle {
            material: background.0.clone(),
            sprite: Sprite {
                size: Vec2::new(200.0, 200.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BackgroundPass)
        .remove::<MainPass>();

    /* Define camera as OrthographicCameraBundle 2d camera with name BACKGROUND_PASS_CAMERA
       Add as active camera and spawn
    */
    let background_pass_camera = OrthographicCameraBundle {
        camera: Camera {
            name: Some(BACKGROUND_PASS_CAMERA.to_string()),
            ..Default::default()
        },
        ..OrthographicCameraBundle::new_2d()
    };
    active_cameras.add(BACKGROUND_PASS_CAMERA);

    commands.spawn_bundle(background_pass_camera);
}

/* Set up our Background pass here */
/* I should be able to reuse the sprite render pipeline (?) */
pub struct BackgroundPass;

pub const BACKGROUND_PASS: &str = "background_pass";
pub const BACKGROUND_PASS_CAMERA: &str = "background_pass_camera";

fn add_background_graph(graph: &mut RenderGraph, msaa: &Msaa) {
    /* Define a PassNode on the BackgroundPass */
    let mut pass_node = PassNode::<&BackgroundPass>::new(PassDescriptor {
        color_attachments: vec![msaa.color_attachment_descriptor(
            TextureAttachment::Input("color_attachment".to_string()),
            TextureAttachment::Input("color_resolve_target".to_string()),
            Operations {
                load: LoadOp::Load,
                store: true,
            },
        )],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
            attachment: TextureAttachment::Input("depth".to_string()),
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }),
        sample_count: msaa.samples,
    });

    /* Add our camera with name BACKGROUND_PASS_CAMERA to the BackgroundPass PassNode */
    pass_node.add_camera(BACKGROUND_PASS_CAMERA);

    /* Add the BackgroundPass PassNode to the RenderGraph with name BACKGROUND_PASS */
    graph.add_node(BACKGROUND_PASS, pass_node);

    /* Connect SpawnChain and MainDepthTexture to our BackgroundPass PassNode */
    graph
        .add_slot_edge(
            base::node::PRIMARY_SWAP_CHAIN,
            WindowSwapChainNode::OUT_TEXTURE,
            BACKGROUND_PASS,
            if msaa.samples > 1 {
                "color_resolve_target"
            } else {
                "color_attachment"
            },
        )
        .unwrap();
    graph
        .add_slot_edge(
            base::node::MAIN_DEPTH_TEXTURE,
            WindowTextureNode::OUT_TEXTURE,
            BACKGROUND_PASS,
            "depth",
        )
        .unwrap();
    if msaa.samples > 1 {
        graph
            .add_slot_edge(
                base::node::MAIN_SAMPLED_COLOR_ATTACHMENT,
                WindowSwapChainNode::OUT_TEXTURE,
                BACKGROUND_PASS,
                "color_attachment",
            )
            .unwrap();
    }

    /* Make sure to put our BackgroundPass PassNode before the MainPass PassNode */
    graph
        .add_node_edge(BACKGROUND_PASS, base::node::MAIN_PASS)
        .unwrap();

    /* Pull in other nodes needs we need to draw a SpriteBundle (?) */
    graph.add_node_edge("transform", BACKGROUND_PASS).unwrap();
    graph.add_node_edge("sprite", BACKGROUND_PASS).unwrap();
    graph
        .add_node_edge("color_material", BACKGROUND_PASS)
        .unwrap();

    /* Add a system Node for our camera and connect it to the Background Pass */
    graph.add_system_node(
        BACKGROUND_PASS_CAMERA,
        CameraNode::new(BACKGROUND_PASS_CAMERA),
    );
    graph
        .add_node_edge(BACKGROUND_PASS_CAMERA, BACKGROUND_PASS)
        .unwrap();
}
