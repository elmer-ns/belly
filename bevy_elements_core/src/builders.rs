use std::cell::RefCell;
use bevy::utils::HashMap;
use std::rc::Rc;
use std::ops::Deref;

use bevy::ecs::system::BoxedSystem;
use bevy::prelude::*;
use crate::context::*;
use crate::element::*;
use crate::tags::*;

pub (crate) fn build_element(mut ctx: ResMut<BuildingContext>, mut commands: Commands) {
    commands
        .entity(ctx.element)
        .insert_bundle(NodeBundle {
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .push_children(&ctx.content());
}

#[derive(Default)]
pub (crate) struct DefaultFont(Handle<Font>);

pub (crate) fn setup_default_font(mut fonts: ResMut<Assets<Font>>, mut default_font: ResMut<DefaultFont>) {
    let default_font_bytes = include_bytes!("SourceCodePro-Light.ttf").to_vec();
    let default_font_asset = Font::try_from_bytes(default_font_bytes).unwrap();
    let default_font_handle = fonts.add(default_font_asset);
    default_font.0 = default_font_handle;
}

pub (crate) fn build_text(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands,
    default_font: Res<DefaultFont>,
) {
    let ctx = ctx.text();
    commands
        .entity(ctx.element)
        .insert_bundle(TextBundle::from_section(
            ctx.text,
            TextStyle {
                font: default_font.0.clone(),
                font_size: 24.0,
                color: Color::WHITE,
            },
        ))
        .insert(Element {
            display: DisplayElement::Inline,
            ..default()
        });
}

pub (crate) fn default_postprocessor(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands,
    mut elements: Query<&mut Element>,
) {
    let tag = ctx.name;
    let element = ctx.element;
    let mut commands = commands.entity(element.clone());
    let mut params = ctx.params();
    commands.insert(Name::new(tag.to_string()));
    
    if let Some(attr_commands) = params.commands(tags::with()) {
        attr_commands(&mut commands);
    }
    if let Ok(mut element) = elements.get_mut(element) {
        element.name = tag;
        element.id = params.id();
        element.classes.extend(params.classes());
        element.styles.extend(params.styles());
        println!("element {} classes:{:?}", element.name, element.classes);
    } else {
        let element = Element {
            name: tag,
            id: params.id(),
            classes: params.classes(),
            styles: params.styles(),
            ..default()
        };
        println!("element {} classes:{:?}", element.name, element.classes);
        commands.insert(element);
    }
}

#[derive(Clone)]
pub struct ElementBuilder {
    system: Rc<RefCell<BoxedSystem<(), ()>>>,
    postprocess: bool
}

impl ElementBuilder {
    pub (crate) fn new<Params, S:IntoSystem<(), (), Params>>(world: &mut World, builder: S) -> ElementBuilder {
        let mut system = IntoSystem::into_system(builder);
        system.initialize(world);
        ElementBuilder {
            postprocess: false,
            system: Rc::new(RefCell::new(Box::new(system)))
        }
    }
    pub(crate) fn with_postprocessing(mut self) -> Self {
        self.postprocess = true;
        self
    }
    pub fn build(&self, world: &mut World) {
        let mut system = self.system.borrow_mut();
        system.run((), world);
        system.apply_buffers(world);
        if !self.postprocess {
            return;
        }
        let processors = world.get_resource_mut::<ElementPostProcessors>().unwrap().0.clone();
        for postprocessor in processors.borrow().iter() {
            postprocessor.build(world)
        }
    }
}

#[derive(Clone)]
pub struct TextElementBuilder(pub (crate) ElementBuilder);
unsafe impl Send for TextElementBuilder {}
unsafe impl Sync for TextElementBuilder {}
impl Deref for TextElementBuilder {
    type Target = ElementBuilder;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct ElementPostProcessors(pub (crate) Rc<RefCell<Vec<ElementBuilder>>>);
unsafe impl Send for ElementPostProcessors {}
unsafe impl Sync for ElementPostProcessors {}


pub struct ElementBuilderRegistry(HashMap<Tag, ElementBuilder>);

unsafe impl Send for ElementBuilderRegistry {}
unsafe impl Sync for ElementBuilderRegistry {}

impl ElementBuilderRegistry {
    pub fn new() -> ElementBuilderRegistry {
        ElementBuilderRegistry(Default::default())
    }

    pub fn get_builder(&self, name: Tag) -> Option<ElementBuilder> {
        self.0.get(&name).map(|b| b.clone())
    }

    pub fn add_builder(&mut self, name: Tag, builder: ElementBuilder) {
        self.0.insert(name, builder);
    }
}
