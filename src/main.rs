mod graph;

fn main() {
    /*
    let mut g = Graph::new(1);

    let mut a = Graph::new(2);
    let b = Graph::new(3);
    let mut c = Graph::new(4);
    let d = Graph::new(5);

    c.add(b);
    c.add(d);
    a.add(c);
    g.add(a);

    std::fs::write(
        "res/input.json",
        g.write_to_str(&mut JsonGraphWriter).unwrap(),
    )
    .unwrap();
    */

    /*
    let g = graph::Graph::from_reader(
        &mut graph::JsonGraphReader,
        &std::fs::read_to_string("res/input.json").unwrap(),
    ).unwrap();

    println!("{}", g.write_to_str(&mut graph::GraphIncidenceMatrixWriter).unwrap());
    */

    let g = graph::Graph::from_reader(
        &mut graph::JsonGraphReader,
        &std::fs::read_to_string("res/input.json").unwrap(),
    ).unwrap();

    println!("{}", g.to_string());
}


