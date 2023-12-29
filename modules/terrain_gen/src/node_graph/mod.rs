use std::{any::TypeId, sync::Arc, borrow::Cow, ascii::escape_default};

use bevy::{prelude::*, reflect::GetTypeRegistration};
use bevy_inspector_egui::egui;
use egui_node_graph::*;
use space_editor_ui::{EditorUiAppExt, editor_tab::{EditorTabName, EditorTab}};

pub struct NodeGraphPlugin;

impl Plugin for NodeGraphPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(EditorTabName::Other("Node Graph".to_string()), GraphTab::default());
    }
}

#[derive(Default, Resource)]
pub struct GraphTab {
    pub selected : Option<Entity>,
}

impl EditorTab for GraphTab {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        if self.selected.is_some() {

        } else {

        }

    }

    fn title(&self) -> egui::WidgetText {
        "Node Graph".into()
    }
}

pub struct Array2d<T> {
    pub data: Vec<Vec<T>>,
    width: usize,
    height: usize,
}

impl<T : Default> Array2d<T> {
    pub fn new(width: usize, height: usize) -> Self {
        let mut data = Vec::with_capacity(height);
        for _ in 0..height {
            let mut sub_data = Vec::with_capacity(width);
            for _ in 0..width {
                sub_data.push(T::default());
            }
            data.push(sub_data);
        }
        Self {
            data,
            width,
            height,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.data[y][x]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        &mut self.data[y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.data[y][x] = value
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;

        self.data.resize_with(height, || {
            let mut sub_data = Vec::with_capacity(width);
            for _ in 0..width {
                sub_data.push(T::default());
            }
            sub_data
        });
    }

    pub fn convert<A>(&self, f: impl Fn(&T) -> A) -> Array2d<A> {
        Array2d {
            data: self.data.iter().map(|row| {
                row.iter().map(|v| f(v)).collect()
            }).collect(),
            width: self.width,
            height: self.height,
        }
    }
}

impl<T: Clone> Clone for Array2d<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            width: self.width,
            height: self.height,
        }
    }
}

//from
impl<'a, T, A> From<&'a Array2d<T>> for Array2d<A> where A : From<&'a T> {
    fn from(array: &'a Array2d<T>) -> Self {
        Array2d {
            data: array.data.iter().map(|row| {
                row.iter().map(|v| A::from(v)).collect()
            }).collect(),
            width: array.width,
            height: array.height,
        }
    }
}

pub trait GraphValue : Reflect {
    fn color(&self) -> egui::Color32;
    fn name(&self) -> String;
}

pub struct ValueHolder {
    pub value: Box<dyn GraphValue>,
}



#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    SetActiveNode(NodeId),
    ClearActiveNode,
}

pub struct GraphState {
    pub active_node: Option<NodeId>,
}

#[derive(Component, Reflect)]
pub struct TerrainGraph {
    #[reflect(ignore)]
    pub graph_editor_state: GraphEditorState<(), String, Box<dyn GraphValue + Send + Sync + 'static>, Box<dyn TerrainNode + Send + Sync + 'static>, ()>
}

pub trait TerrainNode {
    fn clone_value(&self) -> Box<dyn TerrainNode>;
    fn name(&self) -> Cow<str>;

    fn build_node_boxed(
        &self,
        graph: &mut Graph<(), String, Box<dyn GraphValue>>,
        user_state: &GraphState,
        node_id: NodeId,
    );

}

impl Clone for Box<dyn TerrainNode> {
    fn clone(&self) -> Self {
        self.clone_value()
    }

}


impl NodeTemplateTrait for Box<dyn TerrainNode> {
    type NodeData = ();

    type DataType = String;

    type ValueType = Box<dyn GraphValue>;

    type UserState = GraphState;

    fn node_finder_label(&self, user_state: &mut Self::UserState) -> std::borrow::Cow<str> {
        self.name()
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, user_state: &mut Self::UserState) -> Self::NodeData {
        ()
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        self.build_node_boxed(graph, user_state, node_id);
    }
}