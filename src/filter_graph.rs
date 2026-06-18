extern crate ffmpeg_next as ffmpeg;

use ffmpeg::filter::Graph as AvFilterGraph;

use crate::error::Error;

type Result<T> = std::result::Result<T, Error>;

// avfilter_graph_alloc

pub struct FilterGraphBuilder {

}

impl FilterGraphBuilder {
    pub fn new() -> FilterGraphBuilder {
        FilterGraphBuilder {  }
    }

    pub fn build(self) -> Result<FilterGraph> {
        let mut filter = AvFilterGraph::new();

        filter.validate()?;
    }
}

pub struct FilterGraph {
    graph: AvFilterGraph
}

impl FilterGraph {
    pub fn new() -> Result<FilterGraph> {
        todo!();
    }

    pub fn preset_video_normalizer() -> Result<FilterGraph> {
        todo!()
    }

    pub fn dump(&self) -> String {
        self.graph.dump()
    }
}