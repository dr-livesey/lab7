#![allow(dead_code)]

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[cfg(test)]
use mockall::{automock, mock, predicate::*};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Graph {
    value: u8,
    nodes: Vec<Graph>,
}

impl Graph {
    pub fn new(value: u8) -> Self {
        Self {
            value,
            nodes: vec![],
        }
    }

    pub fn add(&mut self, g: Graph) -> &mut Self {
        self.nodes.push(g);

        self
    }

    pub fn from_reader<Reader: GraphReader>(reader: &mut Reader, src: &str) -> Result<Self> {
        reader.read(src)
    }

    pub fn write_to_str<Writer: GraphWriter>(&self, writer: &mut Writer) -> Result<String> {
        writer.write(self)
    }
}

impl ToString for Graph {
    fn to_string(&self) -> String {
        let mut result = format!("{} {{ ", self.value);

        self.nodes
            .iter()
            .for_each(|node| result.push_str(&node.to_string()));
        result.push_str("} ");

        result
    }
}

#[cfg_attr(test, automock)]
pub trait GraphReader {
    fn read(&mut self, src: &str) -> Result<Graph>;
}

#[cfg_attr(test, automock)]
pub trait GraphWriter {
    fn write(&mut self, graph: &Graph) -> Result<String>;
}

pub struct JsonGraphReader;
impl GraphReader for JsonGraphReader {
    fn read(&mut self, src: &str) -> Result<Graph> {
        serde_json::from_str(src).map_err(|err| anyhow!("{}", err.to_string()))
    }
}

pub struct JsonGraphWriter;
impl GraphWriter for JsonGraphWriter {
    fn write(&mut self, graph: &Graph) -> Result<String> {
        serde_json::to_string_pretty(graph).map_err(|err| anyhow!("{}", err.to_string()))
    }
}

#[derive(Debug)]
pub struct IncidenceMatrix {
    header: Vec<String>,
    raw: Vec<Vec<bool>>,
}

impl IncidenceMatrix {
    pub fn new(g: &Graph) -> Self {
        let header = Self::get_header_recursively(g);
        let raw = Self::get_raw_recursively(g, &header);

        Self { header, raw }
    }

    fn get_header_recursively(g: &Graph) -> Vec<String> {
        let mut result: Vec<String> = g
            .nodes
            .iter()
            .map(|node| format!("{}-{}", g.value, node.value))
            .collect();

        for node in &g.nodes {
            result.append(&mut Self::get_header_recursively(&node));
        }

        result
    }

    fn get_raw_recursively(g: &Graph, header: &[String]) -> Vec<Vec<bool>> {
        // first we need to collect all graphs references into one list
        // then header value starts with the graph vertex value

        let values = Self::get_vertices_list(g);

        let mut result = vec![];

        for i in 0..values.len() {
            result.push(vec![]);
            for column in header {
                result[i].push(column.starts_with(&values[i].to_string()))
            }
        }

        result
    }

    // I think that better to return vertices values as is, not through
    // their containers
    fn get_vertices_list(g: &Graph) -> Vec<u8> {
        let mut result = vec![];

        result.push(g.value);
        g.nodes
            .iter()
            .for_each(|node| result.append(&mut Self::get_vertices_list(node)));

        result
    }
}

pub struct GraphIncidenceMatrixWriter;
impl GraphWriter for GraphIncidenceMatrixWriter {
    fn write(&mut self, graph: &Graph) -> Result<String> {
        Ok(format!("{:#?}", IncidenceMatrix::new(graph)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubGraphWriter {
        pub ok: bool,
    }
    impl GraphWriter for StubGraphWriter {
        fn write(&mut self, _: &Graph) -> Result<String> {
            self.ok = true;
            Err(anyhow!("Should return error by design, but not throw"))
        }
    }

    struct StubGraphReader {
        pub ok: bool,
    }
    impl GraphReader for StubGraphReader {
        fn read(&mut self, _: &str) -> Result<Graph> {
            self.ok = true;
            Err(anyhow!("Should return error by design, but not throw"))
        }
    }

    #[test]
    fn stub_test_reader() {
        let mut reader = StubGraphReader { ok: false };
        let _ = Graph::from_reader(&mut reader, "");

        assert!(reader.ok)
    }

    #[test]
    fn mock_test_reader() {
        let mut mock = MockGraphReader::new();
        mock.expect_read().returning(|_| Ok(fill_the_graph()));

        let g = fill_the_graph();

        assert_eq!(mock.read("anything").unwrap(), g);
    }

    #[test]
    fn stub_test_writer() {
        let mut writer = StubGraphWriter { ok: false };
        let _ = Graph::new(0).write_to_str(&mut writer);

        assert!(writer.ok)
    }

    #[test]
    fn mock_test_writer() {
        let mut mock = MockGraphWriter::new();
        mock.expect_write()
            .returning(|_| GraphIncidenceMatrixWriter {}.write(&fill_the_graph()));

        assert_eq!(
            mock.write(&fill_the_graph()).unwrap(),
            r##"IncidenceMatrix {
    header: [
        "1-2",
        "2-4",
        "4-3",
        "4-5",
    ],
    raw: [
        [
            true,
            false,
            false,
            false,
        ],
        [
            false,
            true,
            false,
            false,
        ],
        [
            false,
            false,
            true,
            true,
        ],
        [
            false,
            false,
            false,
            false,
        ],
        [
            false,
            false,
            false,
            false,
        ],
    ],
}"##
        );
    }

    fn fill_the_graph() -> Graph {
        let mut g = Graph::new(1);

        let mut a = Graph::new(2);
        let b = Graph::new(3);
        let mut c = Graph::new(4);
        let d = Graph::new(5);

        c.add(b);
        c.add(d);
        a.add(c);
        g.add(a);

        g
    }

    #[test]
    fn graph_to_string_test() {
        let g = fill_the_graph();

        assert_eq!(g.to_string(), "1 { 2 { 4 { 3 { } 5 { } } } } ")
    }

    #[test]
    fn get_graph_from_json() {
        let g = fill_the_graph();

        assert_eq!(g, Graph::from_reader(&mut JsonGraphReader, r##"{"value":1,"nodes":[{"value":2,"nodes":[{"value":4,"nodes":[{"value":3,"nodes":[]},{"value":5,"nodes":[]}]}]}]}"##).unwrap())
    }
}
