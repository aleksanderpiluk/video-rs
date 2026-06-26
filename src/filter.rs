extern crate ffmpeg_next as ffmpeg;

mod buffer;
mod buffer_sink;
mod hflip;
mod null_sink;
mod null_src;
mod overlay;
mod rotate;
mod scale;
mod split;
mod transpose;
mod vflip;

use std::{cell::RefCell, fmt::Debug};

pub use buffer::{BufferFilter, BufferFilterHandle};
pub use buffer_sink::{BufferSinkFilter, BufferSinkFilterHandle};
pub use hflip::{HFlipFilter, HFlipFilterHandle};
pub use null_sink::{NullSinkFilter, NullSinkFilterHandle};
pub use null_src::{NullSrcFilter, NullSrcFilterHandle};
pub use overlay::{OverlayFilter, OverlayFilterHandle};
pub use rotate::{RotateFilter, RotateFilterHandle};
pub use scale::{ScaleFilter, ScaleFilterHandle};
pub use split::{SplitFilter, SplitFilterHandle};
pub use transpose::{TransposeDir, TransposeFilter, TransposeFilterHandle, TransposePassthrough};
pub use vflip::{VFlipFilter, VFlipFilterHandle};

use ffmpeg::filter::{Context as AVContext, Graph as AvFilterGraph};

use crate::{Error, frame::RawFrame};

type Result<T> = std::result::Result<T, Error>;

pub struct FilterGraphBuilder {
    state: FilterGraphBuilderState,
}

impl FilterGraphBuilder {
    pub fn new() -> Self {
        Self {
            state: FilterGraphBuilderState {
                nodes: vec![],
                in_pads: vec![],
                out_pads: vec![],
            },
        }
    }

    pub fn add_filters(mut self, f: impl FnOnce(&mut FilterGraphBuilderScope<'_>) -> ()) -> Self {
        let state = RefCell::new(self.state);
        f(&mut FilterGraphBuilderScope { state: &state });

        self.state = state.into_inner();
        self
    }

    pub fn build(self) -> Result<FilterGraph> {
        let mut graph = AvFilterGraph::new();

        let mut av_contexts: Vec<Option<AVContext>> = (0..self.state.nodes.len())
            .into_iter()
            .map(|_| None)
            .collect();
        for (filter_id, filter) in self.state.nodes.iter().enumerate() {
            let mut ctx = SynthesisContext {
                filter_id,
                av_graph: &mut graph,
                av_contexts: &mut av_contexts,
            };
            filter.synthesize(&mut ctx);
        }

        for out_pad in self.state.out_pads {
            if let Some(link) = out_pad.link {
                let in_pad = &self.state.in_pads[link.0];
                let [Some(out_filter), Some(in_filter)] = av_contexts
                    .get_disjoint_mut([out_pad.filter_id, in_pad.filter_id])
                    .unwrap()
                else {
                    todo!()
                };
                out_filter.link(out_pad.index, in_filter, in_pad.index);
            }
        }
        
        graph.validate()?;

        Ok(FilterGraph { graph })
    }
}

#[derive(Debug)]
pub struct FilterGraphBuilderState {
    nodes: Vec<Box<dyn Synthesizeable>>,
    in_pads: Vec<InPad>,
    out_pads: Vec<OutPad>,
}

pub struct FilterGraphBuilderScope<'a> {
    state: &'a RefCell<FilterGraphBuilderState>,
}

impl<'a> FilterGraphBuilderScope<'a> {
    pub fn add_filter<T: Filter<U> + 'static, U: 'static>(&mut self, filter_builder: T) -> U {
        let mut state = self.state.borrow_mut();

        let filter_id = state.nodes.len();
        let mut filter_ctx = NodeContext {
            graph_builder: &mut state,
            filter_id,
        };
        let (filter, handle) = filter_builder.register(&mut filter_ctx);
        state.nodes.push(filter);

        handle
    }

    pub fn link(&mut self, output: OutPadHandle, input: InPadHandle) {
        let mut state = self.state.borrow_mut();
        state.out_pads[output.0].link = Some(input);
        state.in_pads[input.0].link = Some(output);
    }
}

trait Registerable {
    type Handle;

    fn register(self, ctx: &mut NodeContext) -> (Box<dyn Synthesizeable>, Self::Handle);
}

trait Synthesizeable: Debug {
    fn synthesize(&self, ctx: &mut SynthesisContext);
}

pub(crate) trait Filter<T>: Registerable<Handle = T> + Synthesizeable {}

pub(crate) struct NodeContext<'a> {
    graph_builder: &'a mut FilterGraphBuilderState,
    filter_id: usize,
}

impl NodeContext<'_> {
    pub(crate) fn new_in_pad(&mut self, index: u32) -> InPadHandle {
        let id = self.graph_builder.in_pads.len();
        self.graph_builder
            .in_pads
            .push(InPad::new(self.filter_id, index));

        InPadHandle(id)
    }

    pub(crate) fn new_out_pad(&mut self, index: u32) -> OutPadHandle {
        let id = self.graph_builder.out_pads.len();
        self.graph_builder
            .out_pads
            .push(OutPad::new(self.filter_id, index));

        OutPadHandle(id)
    }
}

struct SynthesisContext<'a> {
    filter_id: usize,
    av_graph: &'a mut AvFilterGraph,
    av_contexts: &'a mut Vec<Option<AVContext>>,
}

impl<'a> SynthesisContext<'a> {
    pub fn add_filter(&mut self, name: &str, id: &str) -> AVFilterBuilder<'_, 'a> {
        AVFilterBuilder::new(self, name, id)
    }
}

pub struct FilterGraph {
    graph: AvFilterGraph,
}

impl FilterGraph {
    pub fn dump(&self) -> String {
        self.graph.dump()
    }

    pub fn source(&mut self, name: &str) -> Source {
        Source {
            context: self.graph.get(name).unwrap(),
        }
    }

    pub fn sink(&mut self, name: &str) -> Sink {
        Sink {
            context: self.graph.get(name).unwrap(),
        }
    }
}

pub struct Sink {
    context: AVContext,
}

impl Sink {
    pub fn pull_frame_raw(&mut self) -> RawFrame {
        let mut frame = RawFrame::empty();
        self.context.sink().frame(&mut frame);
        frame
    }
}

pub struct Source {
    context: AVContext,
}

impl Source {
    pub fn push_frame_raw(&mut self, frame: &RawFrame) {
        self.context.source().add(frame);
    }
}

#[derive(Debug)]
pub struct InPad {
    filter_id: usize,
    index: u32,
    link: Option<OutPadHandle>,
}

impl InPad {
    pub fn new(filter_id: usize, index: u32) -> Self {
        Self {
            filter_id,
            index,
            link: None,
        }
    }
}

#[derive(Debug)]
pub struct OutPad {
    filter_id: usize,
    index: u32,
    link: Option<InPadHandle>,
}

impl OutPad {
    pub fn new(filter_id: usize, index: u32) -> Self {
        Self {
            filter_id,
            index,
            link: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InPadHandle(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct OutPadHandle(pub usize);

pub struct AVFilterBuilder<'a, 'b> {
    synthesis_ctx: &'a mut SynthesisContext<'b>,
    name: String,
    id: String,
    args: String,
}

impl<'a, 'b> AVFilterBuilder<'a, 'b> {
    fn new(
        synthesis_ctx: &'a mut SynthesisContext<'b>,
        name: impl Into<String>,
        id: impl Into<String>,
    ) -> Self {
        Self {
            synthesis_ctx,
            name: name.into(),
            id: id.into(),
            args: String::new(),
        }
    }
}

impl AVFilterBuilder<'_, '_> {
    fn add_positional_param(mut self, value: &str) -> Self {
        if self.args.len() > 0 {
            self.args += ":";
        }
        self.args += value;

        self
    }

    fn add_param(mut self, name: &str, value: &str) -> Self {
        if self.args.len() > 0 {
            self.args += ":";
        }
        self.args += &format!("{}={}", name, value);

        self
    }

    fn build(self) {
        let av_context = self
            .synthesis_ctx
            .av_graph
            .add(
                &ffmpeg::filter::find(&self.name).unwrap(),
                &self.id,
                &self.args,
            )
            .unwrap();

        self.synthesis_ctx.av_contexts[self.synthesis_ctx.filter_id] = Some(av_context);
    }
}
