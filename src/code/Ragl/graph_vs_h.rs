#![allow(non_snake_case)]

// ////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Graph
// -----
// A Graph object is one of the most generic data structures.  Theoretically, a graph
// could be used to make a tree, list, or any other standard structure.  A graph has
// two data types: NODE and EDGE, and maintains connection information how edges can
// link two nodes.
//
// One of the most intuitive uses of a graph class is a navigation bouy system.  In such
// a use, the nodes would probably be vector based objects with positional information
// and the edges might contain portal information (door is open / closed / locked).
//
// Another example might be a web page, or decision tree, or any other problem space
// which requires object connection data.
//
//
//
//
// Implimentation
// --------------
// This template allocates a pool for NODES, a pool for EDGES, and a grid2_vs<int> to serve
// as an Adjacency Matrix (called Links).  The Adj. Matrix stores indicies to EDGE objects
// in the EDGE pool.
//
//
//
//
// What If You Do Not Need Any Edge Objects?
// -----------------------------------------
// It's fairly common to have a graph with no connection information other than the
// existance of the link.  For this case, you should be able to create a graph with a
// MAXEDGES of 1.  You will want to call the version of connect_node() which does not
// take an edge object, and uses 1 as the "index" in the Adj. Matrix.
//
//
//
//
// How Do You Search?
// -------------------
// This graph supports 3 search methods:
//  Breadth First - Exausts as many links close to start as possible
//  Depth First	  - Gets as far from start as quickly as possible
//  A*            - Uses a distance heuristic toward end point
//
// First, create a (graph_vs::search) object with the start and end points that you want
// to search for.  Then, call either bfs(), dfs(), or astar().  When you get the
// object back, it will have a vector of all the nodes that were visited and methods
// for iterating over that vector to get the path.
//
//	for (TestSearch.path_begin(); !TestSearch.path_end(); TestSearch.path_inc())
//	{
//		sprintf(Buf, "(%d)", TestSearch.path_at());
//		OutputDebugString(Buf);
//	}
//
//
//
//
// Complexity Analisis
// -------------------
// All data operations except remove_node() are O(1) constant time.
// remove_node() can be O(n) where n is the number of NODES in the graph.
//
// Search routines:
//  BFS -
//  DFS -
//  A*  -
//
//
//
//
// ////////////////////////////////////////////////////////////////////////////////////////

use core::ffi::c_int;

// TODO: Import from actual ratl modules when available
// Local stubs for type dependencies from ratl
pub mod ratl_stub {
    use core::marker::PhantomData;

    pub trait ratl_base {}

    pub struct pool_vs<T, const MAXSIZE: usize> {
        _phantom: PhantomData<T>,
    }
    impl<T, const MAXSIZE: usize> ratl_base for pool_vs<T, MAXSIZE> {}

    pub struct vector_vs<T, const MAXSIZE: usize> {
        _phantom: PhantomData<T>,
    }
    impl<T, const MAXSIZE: usize> ratl_base for vector_vs<T, MAXSIZE> {}

    pub struct array_vs<T, const MAXSIZE: usize> {
        _phantom: PhantomData<T>,
    }
    impl<T, const MAXSIZE: usize> ratl_base for array_vs<T, MAXSIZE> {}

    pub struct bits_vs<const MAXSIZE: usize> {}
    impl<const MAXSIZE: usize> ratl_base for bits_vs<MAXSIZE> {}

    pub struct queue_vs<T, const MAXSIZE: usize> {
        _phantom: PhantomData<T>,
    }
    impl<T, const MAXSIZE: usize> ratl_base for queue_vs<T, MAXSIZE> {}

    pub struct stack_vs<T, const MAXSIZE: usize> {
        _phantom: PhantomData<T>,
    }
    impl<T, const MAXSIZE: usize> ratl_base for stack_vs<T, MAXSIZE> {}

    pub struct grid2_vs<T, const CELLSX: usize, const CELLSY: usize> {
        _phantom: PhantomData<T>,
    }
    impl<T, const CELLSX: usize, const CELLSY: usize> ratl_base for grid2_vs<T, CELLSX, CELLSY> {}
}

pub use ratl_stub as ratl;

// ////////////////////////////////////////////////////////////////////////////////////////
// The Graph Class
// ////////////////////////////////////////////////////////////////////////////////////////
pub struct graph_vs<TNODE, const MAXNODES: usize, TEDGE, const MAXEDGES: usize, const MAXNODENEIGHBORS: usize>
where
    TNODE: Clone,
    TEDGE: Clone,
{
    // ////////////////////////////////////////////////////////////////////////////////////
    // The Graph User Class
    //
    // When executing a search (in particular an A* search), you may want to derive your
    // own user class so that you can provide specific functionality to the search.  For
    // example, you might want characters on one team to bias the cost of going to nodes
    // which are occupied by the other team.  Or you might want to allow specific
    // characters to access some edges where others cannot.  Perhaps one can fly or has
    // a special key to a door...
    //
    // ////////////////////////////////////////////////////////////////////////////////////
    _phantom: core::marker::PhantomData<(TNODE, TEDGE)>,
}

impl<TNODE, const MAXNODES: usize, TEDGE, const MAXEDGES: usize, const MAXNODENEIGHBORS: usize>
    graph_vs<TNODE, MAXNODES, TEDGE, MAXEDGES, MAXNODENEIGHBORS>
where
    TNODE: Clone,
    TEDGE: Clone,
{
    pub trait user {
        // ////////////////////////////////////////////////////////////////////////////////////
        // can be invalid edge (For Region)
        // ////////////////////////////////////////////////////////////////////////////////////
        fn can_be_invalid(&self, edge: &TEDGE) -> bool;

        // ////////////////////////////////////////////////////////////////////////////////////
        // valid edge (For A* and Region)
        // ////////////////////////////////////////////////////////////////////////////////////
        fn is_valid(&self, edge: &mut TEDGE, end_point: c_int) -> bool;

        // ////////////////////////////////////////////////////////////////////////////////////
        // cost estimate from A to B (For A*)
        // ////////////////////////////////////////////////////////////////////////////////////
        fn cost(&self, a: &TNODE, b: &TNODE) -> f32;

        // ////////////////////////////////////////////////////////////////////////////////////
        // cost estimate of Edge (For A*)
        // ////////////////////////////////////////////////////////////////////////////////////
        fn cost_edge(&self, edge: &TEDGE, b: &TNODE) -> f32;

        // ////////////////////////////////////////////////////////////////////////////////////
        // same floor check (For Triangulation)
        // ////////////////////////////////////////////////////////////////////////////////////
        fn on_same_floor(&self, a: &TNODE, b: &TNODE) -> bool;

        // ////////////////////////////////////////////////////////////////////////////////////
        // setup the edge (For Triangulation)
        // ////////////////////////////////////////////////////////////////////////////////////
        fn setup_edge(
            &mut self,
            edge: &mut TEDGE,
            a: c_int,
            b: c_int,
            on_hull: bool,
            node_a: &TNODE,
            node_b: &TNODE,
            can_be_invalid: bool,
        );
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Capacity Enum
    // ////////////////////////////////////////////////////////////////////////////////////
    pub const CAPACITY: usize = MAXNODES;
    pub const NULLEDGE: c_int = -1;

    // Type aliases for the pools and neighbor structures
    type TNodes = ratl::pool_vs<TNODE, MAXNODES>;
    type TEdges = ratl::pool_vs<TEDGE, MAXEDGES>;

    #[repr(C)]
    pub struct SNodeNeighbor {
        pub mEdge: i16,
        pub mNode: i16,
    }

    type TNodeNeighbors = ratl::vector_vs<SNodeNeighbor, MAXNODENEIGHBORS>;
    type TLinks = ratl::array_vs<TNodeNeighbors, MAXNODES>;
    type TGraph = graph_vs<TNODE, MAXNODES, TEDGE, MAXEDGES, MAXNODENEIGHBORS>;

    // ////////////////////////////////////////////////////////////////////////////////////
    // cells class
    // ////////////////////////////////////////////////////////////////////////////////////
    pub struct cells<const NODESPERCELL: usize, const CELLSX: usize, const CELLSY: usize> {
        m_graph: *mut TGraph,
        m_cells: ratl::grid2_vs<SCell<NODESPERCELL>, CELLSX, CELLSY>,
    }

    impl<const NODESPERCELL: usize, const CELLSX: usize, const CELLSY: usize>
        cells<NODESPERCELL, CELLSX, CELLSY>
    {
        pub const SIZEX: usize = CELLSX;
        pub const SIZEY: usize = CELLSY;
        pub const SIZENODES: usize = NODESPERCELL;

        #[repr(C)]
        pub struct SSortNode {
            pub mCost: f32,
            pub mHandle: i16,
        }

        impl PartialOrd for SSortNode {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.mCost.partial_cmp(&other.mCost)
            }
        }

        impl PartialEq for SSortNode {
            fn eq(&self, other: &Self) -> bool {
                self.mCost == other.mCost
            }
        }

        type TSortNodes = ratl::vector_vs<SSortNode, { NODESPERCELL * 25 }>;
        type TCellNodes = ratl::vector_vs<i16, NODESPERCELL>;

        pub struct SCell<const NODESPERCELL: usize> {
            pub mNodes: TCellNodes,
            pub mEdges: TCellNodes,
        }

        type TCells = ratl::grid2_vs<SCell<NODESPERCELL>, CELLSX, CELLSY>;

        // ////////////////////////////////////////////////////////////////////////////////
        //
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn new(g: *mut TGraph) -> Self {
            cells {
                m_graph: g,
                m_cells: ratl::grid2_vs::<SCell<NODESPERCELL>, CELLSX, CELLSY> {
                    _phantom: core::marker::PhantomData,
                },
            }
        }

        pub fn clear(&mut self) {
            // mCells.clear();
            // SCell		EmptyCell;
            // mCells.init(EmptyCell);
        }

        pub fn get_cell_upperleft(&self, x: c_int, y: c_int, x_real: &mut f32, y_real: &mut f32) {
            // mCells.get_cell_upperleft(x,y,xReal,yReal);
        }

        pub fn get_cell_lowerright(&self, x: c_int, y: c_int, x_real: &mut f32, y_real: &mut f32) {
            // mCells.get_cell_lowerright(x,y,xReal,yReal);
        }

        // ////////////////////////////////////////////////////////////////////////////////
        //
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn expand_bounds(&mut self, node_handle: c_int) {
            // TNODE&	node = mGraph.get_node(nodeHandle);
            // mCells.expand_bounds(node[0], node[1]);
        }

        // ////////////////////////////////////////////////////////////////////////////////
        //
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn get_cell(&mut self, x: f32, y: f32) {
            // return mCells.get(x,y);
        }

        pub fn get_cell_int(&mut self, x: c_int, y: c_int) {
            // return mCells.get(x,y);
        }

        pub fn convert_to_cell_coords(&self, x: f32, y: f32, xint: &mut c_int, yint: &mut c_int) {
            // mCells.get_cell_coords(x, y, xint, yint);
        }

        // ////////////////////////////////////////////////////////////////////////////////
        //
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn fill_cells_nodes(&mut self, range: f32) {
            // I. Fill All The Cells With The Points Contained By Those Cells
            //----------------------------------------------------------------
            // bool full = false;
            // for (TNodes::iterator it=mGraph.nodes_begin(); it!=mGraph.nodes_end() && !full; it++)
            // {
            //     TNODE&	node = (*it);
            //     SCell&	cell = mCells.get(node[0], node[1]);
            //
            //     cell.mNodes.push_back(it.index());
            //     full = cell.mNodes.full();
            //     assert(!full || "Cell Filled On Inital Containment"==0);
            // }
            //
            // mCells.scale_by_largest_axis(range);
            //
            //
            // // II. Go To All Neighboring Cells And Get Them
            // //==============================================
            // int					iRange = (int)(range) + 1;
            // TCells::riterator	rcell;
            // TCells::riterator	rcellend;
            // CVec3				cellCenter(0,0,0);
            // CVec3				nodeCenter(0,0,0);
            //
            // TSortNodes			*sortnodes	= new TSortNodes;
            // SSortNode			sortnode;
            //
            // TCells				*sortcells	= new TCells;
            // sortcells->copy_bounds(mCells);
            //
            //
            //
            // // For Every Cell
            // //----------------
            // for (int x=0; x<CELLSX; x++)
            // {
            //     for (int y=0; y<CELLSY; y++)
            //     {
            //         // Setup the Sortnodes vector And Range Iterators
            //         //------------------------------------------------
            //         sortnodes->clear();
            //
            //         mCells.get_cell_position(x,y, cellCenter.v[0], cellCenter.v[1]);
            //
            //         for (rcell = mCells.rangeBegin(iRange,x,y); !rcell.at_end(); rcell++)
            //         {
            //             SCell& cell = (*rcell);
            //
            //             // Add The Nodes To The Sort List
            //             //--------------------------------
            //             for (int i=0; i<cell.mNodes.size() && !sortnodes->full(); i++)
            //             {
            //                 int	nodeHandle = cell.mNodes[i];
            //
            //                 TNODE&	node  = mGraph.get_node(nodeHandle);
            //                 nodeCenter[0] = node[0];
            //                 nodeCenter[1] = node[1];
            //
            //                 sortnode.mHandle	= nodeHandle;
            //                 sortnode.mCost		= cellCenter.Dist2(nodeCenter);
            //                 sortnodes->push_back(sortnode);
            //             }
            //         }
            //
            //         // Sort The Results
            //         //------------------
            //         sortnodes->sort();
            //
            //
            //         // Copy The Sorted Nodes Vector Into The Sorted Cell (Of The Sorted Cell Grid)
            //         //----------------------------------------------------------------------------
            //         SCell&	cell = sortcells->get(x,y);
            //         cell.mNodes.clear();
            //         for (int i=0; i<sortnodes->size() && i<NODESPERCELL; i++)
            //         {
            //             cell.mNodes.push_back((*sortnodes)[i].mHandle);
            //         }
            //     }
            // }
            //
            // // Now Copy The Sorted Results To The Main Cells
            // //-----------------------------------------------
            // for (int xb=0; xb<CELLSX; xb++)
            // {
            //     for (int yb=0; yb<CELLSY; yb++)
            //     {
            //         SCell&	scell = sortcells->get(xb,yb);
            //         SCell&	mcell = mCells.get(xb,yb);
            //         mcell.mNodes = scell.mNodes;
            //     }
            // }
            //
            // delete sortnodes;
            // delete sortcells;
        }

        // ////////////////////////////////////////////////////////////////////////////////
        //
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn fill_cells_edges(&mut self, range: f32) {
            // I. Fill All The Cells With The Points Contained By Those Cells
            //----------------------------------------------------------------
            // bool full = false;
            // for (TEdges::iterator eit=mGraph.edges_begin(); eit!=mGraph.edges_end() && !full; eit++)
            // {
            //     TEDGE&	edge = (*eit);
            //     SCell&	cell = mCells.get(edge[0], edge[1]);
            //
            //     cell.mEdges.push_back(eit.index());
            //     full = cell.mEdges.full();
            //     assert(!full || "Cell Filled On Inital Containment"==0);
            // }
            //
            //
            // mCells.scale_by_largest_axis(range);
            //
            //
            // // II. Go To All Neighboring Cells And Get Them
            // //==============================================
            // int					iRange = (int)(range) + 1;
            // TCells::riterator	rcell;
            // TCells::riterator	rcellend;
            // CVec3				cellCenter(0,0,0);
            // CVec3				nodeCenter(0,0,0);
            //
            // TSortNodes			*sortedges	= new TSortNodes;
            // SSortNode			sortnode;
            //
            // TCells				*sortcells	= new TCells;
            // sortcells->copy_bounds(mCells);
            //
            //
            // // For Every Cell
            // //----------------
            // for (int x=0; x<CELLSX; x++)
            // {
            //     for (int y=0; y<CELLSY; y++)
            //     {
            //         // Setup the Sortnodes vector And Range Iterators
            //         //------------------------------------------------
            //         sortedges->clear();
            //
            //         mCells.get_cell_position(x,y, cellCenter.v[0], cellCenter.v[1]);
            //
            //         for (rcell = mCells.rangeBegin(iRange,x,y); !rcell.at_end(); rcell++)
            //         {
            //             SCell& cell = (*rcell);
            //
            //             // Add The Edges To The Sort List
            //             //--------------------------------
            //             for (int e=0; e<cell.mEdges.size() && !sortedges->full(); e++)
            //             {
            //                 int	edgeHandle = cell.mEdges[e];
            //
            //                 TEDGE&	edge  = mGraph.get_edge(edgeHandle);
            //                 nodeCenter[0] = edge[0];
            //                 nodeCenter[1] = edge[1];
            //
            //                 sortnode.mHandle	= edgeHandle;
            //                 sortnode.mCost		= cellCenter.Dist2(nodeCenter);
            //                 sortedges->push_back(sortnode);
            //             }
            //         }
            //
            //         // Sort The Results
            //         //------------------
            //         sortedges->sort();
            //
            //
            //         // Copy The Sorted Nodes Vector Into The Sorted Cell (Of The Sorted Cell Grid)
            //         //----------------------------------------------------------------------------
            //         SCell&	cell = sortcells->get(x,y);
            //         cell.mEdges.clear();
            //         for (int j=0; j<sortedges->size() && j<NODESPERCELL; j++)
            //         {
            //             cell.mEdges.push_back((*sortedges)[j].mHandle);
            //         }
            //     }
            // }
            //
            // // Now Copy The Sorted Results To The Main Cells
            // //-----------------------------------------------
            // for (int xb=0; xb<CELLSX; xb++)
            // {
            //     for (int yb=0; yb<CELLSY; yb++)
            //     {
            //         SCell&	scell = sortcells->get(xb,yb);
            //         SCell&	mcell = mCells.get(xb,yb);
            //         mcell.mEdges = scell.mEdges;
            //     }
            // }
            //
            // delete sortedges;
            // delete sortcells;
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Remove All Edges
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear_edges(&mut self) {
        // mEdges.clear();
        // mEdges.alloc();		// Alloc a dummy edge at location 0
        // for (int i=0; i<MAXNODES; i++)
        // {
        //     mLinks[i].clear();
        // }
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Clear Out All Nodes And Edges
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self) {
        // mNodes.clear();
        // mNodes.alloc();		// Alloc A dummy node at location 0
        // clear_edges();
        //
        // #if !defined(FINAL_BUILD)
        //     mSearchCount = 0;
        //     mSearchMemorySize = 0;
        //
        //     mSearchSuccess = 0;
        //     mSearchSuccessVisited = 0;
        //     mSearchSuccessPathLen = 0;
        //
        //     mSearchFail = 0;
        //     mSearchFailVisited = 0;
        // #endif
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        let mut graph = graph_vs {
            _phantom: core::marker::PhantomData,
        };
        // graph.clear();
        graph
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Number Of Nodes
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn size_nodes(&self) -> c_int {
        // return (mNodes.size()-1);
        0
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Access A Node
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn get_node(&self, i: c_int) -> &TNODE {
        // return mNodes[i];
        panic!("get_node stub")
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Access The Beginning Of The Nodes List
    // ////////////////////////////////////////////////////////////////////////////////////
    // typename TNodes::iterator	nodes_begin()
    // {
    //     typename TNodes::iterator x = mNodes.begin();
    //     x++;
    //     return x;
    // }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Access The End Of The Nodes List
    // ////////////////////////////////////////////////////////////////////////////////////
    // typename TNodes::iterator	nodes_end()
    // {
    //     return mNodes.end();
    // }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Number Of Edges
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn size_edges(&self) -> c_int {
        // return (mEdges.size()-1);
        0
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Access An Edge
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn get_edge(&self, i: c_int) -> &TEDGE {
        // return mEdges[i];
        panic!("get_edge stub")
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Get The Edge That Connects nodeA to nodeB
    //
    // NOTE: This is now a linear scan witout the adjacency matrix (worst case 8 depth)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn get_edge_across(&self, node_a: c_int, node_b: c_int) -> c_int {
        // int numNeighbors = mLinks[nodeA].size();
        // for (int curNeighbor=0; curNeighbor<numNeighbors; curNeighbor++)
        // {
        //     if (mLinks[nodeA][curNeighbor].mNode==nodeB)
        //     {
        //         if (mLinks[nodeA][curNeighbor].mEdge)
        //         {
        //             return mLinks[nodeA][curNeighbor].mEdge;
        //         }
        //         return -1;	// -1 signifies that a link exists with no edge
        //     }
        // }
        0
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Access The Beginning Of The Nodes List
    // ////////////////////////////////////////////////////////////////////////////////////
    // typename TEdges::iterator	edges_begin()
    // {
    //     typename TEdges::iterator x = mEdges.begin();
    //     x++;
    //     return x;
    // }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Access The End Of The Nodes List
    // ////////////////////////////////////////////////////////////////////////////////////
    // typename TEdges::iterator	edges_end()
    // {
    //     return mEdges.end();
    // }

    pub fn edge_index(&self, edge: &TEDGE) -> c_int {
        // return mEdges.pointer_to_index(&edge);
        0
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Get All The Neighbors Of A Given Node
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn get_node_neighbors(&self, node_a: c_int) -> &ratl::vector_vs<SNodeNeighbor, MAXNODENEIGHBORS> {
        // return mLinks[nodeA];
        panic!("get_node_neighbors stub")
    }

    pub fn node_has_neighbors(&self, node_a: c_int) -> bool {
        // return !(mLinks[nodeA].empty());
        false
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Add A Node
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn insert_node(&mut self, t: &TNODE) -> c_int {
        // int	nNode = mNodes.alloc();
        // mNodes[nNode] = t;
        // return nNode;
        0
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Remove Node And Clear Out All Edges Once Connected To This Node
    //   Note: This is a fairly expensive operation, not to be done often
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn remove_node(&mut self, node: c_int) {
        // mNodes.free(node);
        //
        // // For Each Link To A Neighboring Node
        // //--------------------------------------
        // for (int i=0; i<mLinks[node].size(); i++)
        // {
        //     int	curNeighbor = mLinks[node][i].mNode;
        //     int curEdge		= mLinks[node][i].mEdge;
        //
        //     // Free The Edge
        //     //---------------
        //     if (curEdge)
        //     {
        //         mEdges.free(curEdge);
        //     }
        //
        //
        //     // Remove The Edge From Any Recorded Neighbors
        //     //---------------------------------------------
        //     TNodeNeighbors& neighbors = mLinks[curNeighbor];
        //     for (int j=0; j<neighbors.size(); j++)
        //     {
        //         if (neighbors[j].mNode==node)
        //         {
        //             neighbors.erase_swap(j);
        //             break;
        //         }
        //     }
        // }
        // mLinks[node].clear();
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Connect Node With An Edge Object  (A->B)  if reflexive, also (B->A)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn connect_node(&mut self, t: &TEDGE, node_a: c_int, node_b: c_int, reflexive: bool) -> c_int {
        // if (nodeA==nodeB || !nodeA || !nodeB || !mNodes.is_used(nodeA) || !mNodes.is_used(nodeB))
        // {
        //     assert("ERROR: Cannot Connect A and B!"==0);
        //     return 0;
        // }
        //
        // if (mLinks[nodeA].full() || (reflexive && mLinks[nodeB].full()))
        // {
        //     assert("ERROR: Max edges per node exceeded!"==0);
        //     return 0;
        // }
        //
        // if (mEdges.full())
        // {
        //     assert("ERROR: Max edges exceeded!"==0);
        //     return 0;
        // }
        //
        // SNodeNeighbor	nNbr;
        //
        // nNbr.mNode = nodeB;
        // nNbr.mEdge = mEdges.alloc();
        // mEdges[nNbr.mEdge] = t;
        //
        //
        // mLinks[nodeA].push_back(nNbr);
        // if (reflexive)
        // {
        //     nNbr.mNode = nodeA;
        //     mLinks[nodeB].push_back(nNbr);
        // }
        //
        // return nNbr.mEdge;
        0
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Connect Node Without Allocating An Edge Object  (A->B)  if reflexive, also (B->A)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn connect_node_no_edge(&mut self, node_a: c_int, node_b: c_int, reflexive: bool) {
        // if (nodeA==nodeB || !nodeA || !nodeB || !mNodes.is_used(nodeA) || !mNodes.is_used(nodeB))
        // {
        //     assert("ERROR: Cannot Connect A and B!"==0);
        //     return 0;
        // }
        //
        // if (mLinks[nodeA].full() || (reflexive && mLinks[nodeB].full()))
        // {
        //     assert("ERROR: Max edges per node exceeded!"==0);
        //     return 0;
        // }
        //
        //
        // SNodeNeighbor	nNbr;
        //
        // nNbr.mNode = nodeB;
        // nNbr.mEdge = 0;
        //
        //
        // mLinks[nodeA].push_back(nNbr);
        // if (reflexive)
        // {
        //     nNbr.mNode = nodeA;
        //     mLinks[nodeB].push_back(nNbr);
        // }
        //
        // return nNbr.mEdge;
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Remove Edge
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn remove_edge(&mut self, node_a: c_int, node_b: c_int, reflexive: bool) {
        // if (!mNodes.is_used(nodeA) || !mNodes.is_used(nodeB) && nodeA==nodeB)
        // {
        //     assert("Unable To Remove Edge"==0);
        //     return;
        // }
        //
        // int	linkNum=0;
        //
        // for (linkNum=0; linkNum<mLinks[nodeA].size(); linkNum++)
        // {
        //     if (mLinks[nodeA][linkNum].mNode==nodeB)
        //     {
        //         if (mLinks[nodeA][linkNum].mEdge && mEdges.is_used(mLinks[nodeA][linkNum].mEdge))
        //         {
        //             mEdges.free(mLinks[nodeA][linkNum].mEdge);
        //         }
        //         mLinks[nodeA].erase_swap(linkNum);
        //         break;
        //     }
        // }
        //
        //
        // for (linkNum=0; linkNum<mLinks[nodeB].size(); linkNum++)
        // {
        //     if (mLinks[nodeB][linkNum].mNode==nodeA)
        //     {
        //         if (mLinks[nodeB][linkNum].mEdge && mEdges.is_used(mLinks[nodeB][linkNum].mEdge))
        //         {
        //             mEdges.free(mLinks[nodeB][linkNum].mEdge);
        //         }
        //         mLinks[nodeB].erase_swap(linkNum);
        //         break;
        //     }
        // }
    }

    // ////////////////////////////////////////////////////////////////////////////////////////
    // The Handle Heap Class
    //
    // A handle heap is a specially designed heap class which remembers the handle of any
    // value that is inserted.  Should the value of an already inserted object change, it is
    // possible to reheapify around that value, no matter where the value lies inside the heap
    // because of the additional memory to convert handle->index.
    //
    // The handle heap is used by A* to sort the open list by cost.
    //
    // ////////////////////////////////////////////////////////////////////////////////////////
    pub struct handle_heap<T> {
        m_nodes: *mut TNodes,
        m_data: ratl::array_vs<T, { MAXNODES + 1 }>,
        m_handle_to_pos: ratl::array_vs<c_int, MAXNODES>,
        m_push: c_int,
    }

    impl<T> handle_heap<T>
    where
        T: Clone,
    {
        // ////////////////////////////////////////////////////////////////////////////////////
        // Constructor
        // ////////////////////////////////////////////////////////////////////////////////////
        pub fn new(nodes: *mut TNodes) -> Self {
            let mut heap = handle_heap {
                m_nodes: nodes,
                m_data: ratl::array_vs::<T, { MAXNODES + 1 }> {
                    _phantom: core::marker::PhantomData,
                },
                m_handle_to_pos: ratl::array_vs::<c_int, MAXNODES> {
                    _phantom: core::marker::PhantomData,
                },
                m_push: 0,
            };
            heap.clear();
            heap
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Get The Size (The Difference Between The Push And Pop "Pointers")
        // ////////////////////////////////////////////////////////////////////////////////////
        pub fn size(&self) -> c_int {
            self.m_push
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Check To See If The Size Is Zero
        // ////////////////////////////////////////////////////////////////////////////////////
        pub fn empty(&self) -> bool {
            self.size() == 0
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Check To See If The Size Is Full
        // ////////////////////////////////////////////////////////////////////////////////////
        pub fn full(&self) -> bool {
            self.size() == MAXNODES as c_int
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Empty Out The Entire Heap
        // ////////////////////////////////////////////////////////////////////////////////////
        pub fn clear(&mut self) {
            self.m_push = 0;
            // for (int i=0; i<MAXNODES; i++)
            // {
            //     mHandleToPos[i] = -1;
            // }
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Check If The Handle Has Been Added To This Heap
        // ////////////////////////////////////////////////////////////////////////////////////
        pub fn used(&self, handle: c_int) -> bool {
            // return (mHandleToPos[handle]!=-1 && mData[mHandleToPos[handle]].handle()==handle);
            false
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Get The Data Value At The Top Of The Heap
        // ////////////////////////////////////////////////////////////////////////////////////
        pub fn top(&self) -> &T {
            // assert(mPush>0);		// Don't Try To Look At This If There Is Nothing In Here
            // return (mData[0]);
            panic!("top stub")
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Accessor
        // ////////////////////////////////////////////////////////////////////////////////////
        pub fn get(&self, handle: c_int) -> &T {
            // // If You Hit This Assert, Then You Are Asking For Unallocated Data
            // //------------------------------------------------------------------
            // assert(used(handle));
            // return mData[mHandleToPos[handle]];
            panic!("get stub")
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Add A Value To The Queue
        // ////////////////////////////////////////////////////////////////////////////////////
        pub fn push(&mut self, n_value: &T) {
            // assert(size()<MAXNODES);
            //
            // // Get The Handle From The Value And Make Sure We Don't Already Have One Stored There
            // //------------------------------------------------------------------------------------
            // assert(mHandleToPos[nValue.handle()] == -1);
            //
            // // Add It
            // //--------
            // mData[mPush]										= nValue;
            // mHandleToPos[nValue.handle()]	= mPush;
            //
            //
            //
            // // Fix Possible Heap Inconsistancies
            // //-----------------------------------
            // reheapify_upward(mPush);
            //
            // mPush++;
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Remove A Value From The Queue
        // ////////////////////////////////////////////////////////////////////////////////////
        pub fn pop(&mut self) {
            // assert(size()>0);
            //
            // mPush--;
            //
            // assert(mHandleToPos[mData[0].handle()]==0);
            //
            //
            // // Swap The Lowest Element Up To The Spot We Just "Erased"
            // //---------------------------------------------------------
            // swap(0, mPush);
            //
            // mHandleToPos[mData[mPush].handle()] = -1;	// Erase This Handles Marker
            //
            //
            // // Fix Possible Heap Inconsistancies
            // //-----------------------------------
            // reheapify_downward(0);
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Call This Func If The Value At The Given Handle Has Changed & Needs Adjustment
        // ////////////////////////////////////////////////////////////////////////////////////
        pub fn reheapify(&mut self, handle: c_int) {
            // assert(used(handle));
            //
            // int Pos = mHandleToPos[handle];
            // reheapify_upward(Pos);
            // reheapify_downward(Pos);
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Returns The Location Of Node (i)'s Parent Node (The Parent Node Of Zero Is Zero)
        // ////////////////////////////////////////////////////////////////////////////////////
        fn parent(&self, i: c_int) -> c_int {
            ((i - 1) / 2)
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Returns The Location Of Node (i)'s Left Child (The Child Of A Leaf Is The Leaf)
        // ////////////////////////////////////////////////////////////////////////////////////
        fn left(&self, i: c_int) -> c_int {
            (2 * i) + 1
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Returns The Location Of Node (i)'s Right Child (The Child Of A Leaf Is The Leaf)
        // ////////////////////////////////////////////////////////////////////////////////////
        fn right(&self, i: c_int) -> c_int {
            (2 * i) + 2
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Returns The Location Of Largest Child Of Node (i)
        // ////////////////////////////////////////////////////////////////////////////////////
        fn largest_child(&self, i: c_int) -> c_int {
            // if (left(i)<mPush)
            // {
            //     if (right(i)<mPush)
            //     {
            //         return ( (mData[right(i)] < mData[left(i)]) ? (left(i)) : (right(i)) );
            //     }
            //     return left(i);	// Node i only has a left child, so by default it is the biggest
            // }
            // return i;		// Node i is a leaf, so just return it
            i
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Swaps Two Element Locations
        // ////////////////////////////////////////////////////////////////////////////////////
        fn swap(&mut self, a: c_int, b: c_int) {
            // if (a==b)
            // {
            //     return;
            // }
            //
            // assert(a>=0 && b>=0 && a<MAXNODES && b<MAXNODES);
            // assert(mHandleToPos[mData[a].handle()]==a);
            // assert(mHandleToPos[mData[b].handle()]==b);
            //
            // // Swap Handles
            // //--------------
            // mHandleToPos[mData[a].handle()] = b;
            // mHandleToPos[mData[b].handle()] = a;
            //
            // // Swap Data
            // //-----------
            // mData[MAXNODES]	= mData[a];		// a->TEMP
            // mData[a]		= mData[b];		// b->a
            // mData[b]		= mData[MAXNODES];	// TEMP->B
            //
            // assert(mHandleToPos[mData[a].handle()]==a);
            // assert(mHandleToPos[mData[b].handle()]==b);
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Swaps The Data Up The Heap Until It Reaches A Valid Location
        // ////////////////////////////////////////////////////////////////////////////////////
        fn reheapify_upward(&mut self, _pos: c_int) {
            // while (Pos && mData[parent(Pos)]<mData[Pos])
            // {
            //     swap(parent(Pos), Pos);
            //     Pos = parent(Pos);
            // }
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Swaps The Data Down The Heap Until It Reaches A Valid Location
        // ////////////////////////////////////////////////////////////////////////////////////
        fn reheapify_downward(&mut self, _pos: c_int) {
            // int largestChild = largest_child(Pos);
            // while (largestChild!=Pos && mData[Pos]<mData[largestChild])
            // {
            //     swap(largestChild, Pos);
            //     Pos = largestChild;
            //     largestChild = largest_child(Pos);
            // }
        }

        // ////////////////////////////////////////////////////////////////////////////////////
        // Validate Will Run Through The Heap And Make Sure The Top Element Is Smallest
        // ////////////////////////////////////////////////////////////////////////////////////
        // #ifdef _DEBUG
        //     fn validate(&self) {
        //         for (int i=1; i<mPush; i++)
        //         {
        //             assert(mData[i]<mData[0]);
        //         }
        //     }
        // #endif
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // A Search Node
    // ////////////////////////////////////////////////////////////////////////////////////
    #[repr(C)]
    pub struct search_node {
        pub mNode: c_int,
        pub mParentVisit: c_int,
        pub mCostToGoal: f32,
        pub mCostFromStart: f32,
    }

    impl search_node {
        // ////////////////////////////////////////////////////////////////////////////////
        // Constructors
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn new() -> Self {
            search_node {
                mNode: -1,
                mParentVisit: -1,
                mCostToGoal: -1.0,
                mCostFromStart: 0.0,
            }
        }

        pub fn with_values(node: c_int, parent: c_int) -> Self {
            search_node {
                mNode: node,
                mParentVisit: parent,
                mCostToGoal: -1.0,
                mCostFromStart: 0.0,
            }
        }

        // ////////////////////////////////////////////////////////////////////////////////
        //
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn handle(&self) -> c_int {
            self.mNode
        }

        // ////////////////////////////////////////////////////////////////////////////////
        //
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn cost_estimate(&self) -> f32 {
            self.mCostFromStart + self.mCostToGoal
        }

        // ////////////////////////////////////////////////////////////////////////////////
        //
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn cmp(&self, other: &search_node) -> bool {
            self.cost_estimate() > other.cost_estimate()
        }
    }

    pub type TVisited = ratl::vector_vs<search_node, MAXNODES>;
    pub type TVisitedHandles = ratl::array_vs<c_int, MAXNODES>;
    pub type TNodeState = ratl::bits_vs<MAXNODES>;

    // ////////////////////////////////////////////////////////////////////////////////////
    // The Search Data Object
    // ////////////////////////////////////////////////////////////////////////////////////
    pub struct search {
        pub mStart: c_int,
        pub mEnd: c_int,

        m_nodes_ptr: *mut TNodes,

        m_path_visit: c_int,
        m_prev_index: c_int,
        m_next_index: c_int,
        m_next: search_node,

        m_closed: ratl::bits_vs<MAXNODES>,
        m_visited: TVisited,
        m_node_index_to_visited: TVisitedHandles,
    }

    impl search {
        // NULL constants
        pub const NULL_NODE: c_int = 0;
        pub const NULL_NODE_INDEX: c_int = -1;
        pub const NULL_VISIT_INDEX: c_int = -1;
        pub const NULL_COST: f32 = -1.0;

        // ////////////////////////////////////////////////////////////////////////////////
        // Constructor
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn new(node_start: c_int, node_end: c_int) -> Self {
            let mut search_obj = search {
                mStart: node_start,
                mEnd: node_end,
                m_nodes_ptr: core::ptr::null_mut(),
                m_path_visit: Self::NULL_VISIT_INDEX,
                m_prev_index: Self::NULL_NODE_INDEX,
                m_next_index: Self::NULL_NODE_INDEX,
                m_next: search_node::new(),
                m_closed: ratl::bits_vs::<MAXNODES> {
                    _phantom: core::marker::PhantomData,
                },
                m_visited: ratl::vector_vs::<search_node, MAXNODES> {
                    _phantom: core::marker::PhantomData,
                },
                m_node_index_to_visited: ratl::array_vs::<c_int, MAXNODES> {
                    _phantom: core::marker::PhantomData,
                },
            };
            search_obj.clear(true, false);
            search_obj
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // Reset All The Search Parameters
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn clear(&mut self, clear_nodes_ptr: bool, clear_start_and_end: bool) {
            // Reset All Data
            //----------------
            // mClosed.clear();
            // mVisited.clear();
            // mNodeIndexToVisited.fill(NULL_VISIT_INDEX);
            //
            //
            // mNext.mNode			= NULL_NODE;
            // mNext.mParentVisit	= NULL_VISIT_INDEX;
            // mNext.mCostToGoal	= NULL_COST;
            // mNext.mCostFromStart= NULL_COST;
            //
            // mPrevIndex			= NULL_NODE_INDEX;
            // mNextIndex			= NULL_NODE_INDEX;
            //
            // mPathVisit			= NULL_VISIT_INDEX;

            if clear_nodes_ptr {
                self.m_nodes_ptr = core::ptr::null_mut();
            }

            // Clear Out The Start And End Handles
            //-------------------------------------
            if clear_start_and_end {
                self.mStart = Self::NULL_NODE;
                self.mEnd = Self::NULL_NODE;
            }
            // Otherwise, We Can Start The Next Index
            //----------------------------------------
            else if !self.m_nodes_ptr.is_null() && self.mStart != Self::NULL_NODE && self.mEnd != Self::NULL_NODE {
                self.m_next_index = self.mStart;
            }
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // Call This Function To Clear Out Everything EXCEPT Start And End Points
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn reset(&mut self) {
            self.clear(false, false);
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // Mark This Node As Closed
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn close(&mut self, node: c_int) {
            // mClosed.set_bit(node);
        }

        pub fn close_nodes(&mut self, all_nodes_to_close: &ratl::bits_vs<MAXNODES>) {
            // mClosed |= AllNodesToClose;
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // Return True If We Have Found The Node We Were Searching For
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn success(&self) -> bool {
            if self.mEnd != 0 && self.m_prev_index != Self::NULL_NODE_INDEX {
                return self.m_prev_index == self.mEnd;
            }
            false
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // Get An Index To The Beginning Of The Path
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn path_begin(&mut self) {
            if self.success() {
                // mPathVisit = (mVisited.size()-1);
            } else {
                self.m_path_visit = Self::NULL_VISIT_INDEX;
            }
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // Check ForThe End Of The Path (Use In A for() Loop)
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn path_end(&self) -> bool {
            self.m_path_visit == Self::NULL_VISIT_INDEX
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // Index Path Inc (Get The Old Path Node's Parent)
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn path_inc(&mut self) {
            // assert(mPathVisit!=NULL_VISIT_INDEX);
            // mPathVisit = mVisited[mPathVisit].mParentVisit;
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // Get the node handle of the current node
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn path_at(&self) -> c_int {
            // assert(mPathVisit!=NULL_VISIT_INDEX);
            // return mVisited[mPathVisit].mNode;
            0
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // The Total Cost Of The Path
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn path_cost(&self) -> f32 {
            if self.success() {
                // return (mVisited[(mVisited.size()-1)].mCostFromStart);
            }
            Self::NULL_COST
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // How Many Nodes Were Looked At In This Search
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn num_visited(&self) -> c_int {
            // return mVisited.size();
            0
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // Setup the search data
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn setup(&mut self, nodes_ptr: *mut TNodes) {
            self.m_nodes_ptr = nodes_ptr;
            self.clear(false, false);
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // Pretend the next index is open, probably because we found a shorter route
        // than the first time it was closed, and want it back on the open list
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn reopen_next_index(&mut self) {
            // assert(mNextIndex!=NULL_NODE_INDEX);
            //
            // mNodeIndexToVisited[mNextIndex] = NULL_VISIT_INDEX;
            // mClosed.clear_bit(mNextIndex);
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // The current estimated cost of reaching a give node, if the node was never
        // visited, but IS closed, it returns NULL cost
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn visited_cost(&self, node_index: c_int) -> f32 {
            // int VisitedIndex = mNodeIndexToVisited[NodeIndex];
            // if (VisitedIndex!=NULL_VISIT_INDEX)
            // {
            //     return mVisited[VisitedIndex].cost_estimate();
            // }
            Self::NULL_COST
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // Add This Search Node To The Visited List And Keep Track Of It For Later
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn visit(&mut self, t: &search_node) {
            // assert(mNodesPtr!=0);
            self.m_prev_index = t.mNode;

            // Add It To The Visited List, And Mark The Location In The Node Index Array
            //---------------------------------------------------------------------------
            // mVisited.push_back(t);
            // mNodeIndexToVisited[mPrevIndex] = (mVisited.size()-1);
            // mClosed.set_bit(mPrevIndex);

            // Setup Our Next Node To Know It Came From The Last Location In The Visited Vector
            //----------------------------------------------------------------------------------
            // mNext.mParentVisit				= (mVisited.size()-1);
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // Check To See If The Next Index Has Already Been Closed
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn next_index_closed(&self) -> bool {
            // assert(mNextIndex!=NULL_NODE_INDEX);
            //
            // return (mClosed.get_bit(mNextIndex));
            false
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // The Simple "Get Next" Just Converts The Next Index To A Handle In mNext
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn get_next(&mut self) -> &mut search_node {
            // assert(mNodesPtr);
            // assert(mNextIndex!=NULL_NODE_INDEX);
            //
            // mClosed.set_bit(mNextIndex);
            //
            // mNext.mNode				= mNextIndex;
            // mNext.mCostToGoal		= 0;
            // mNext.mCostFromStart	= 0;
            //
            // return mNext;
            &mut self.m_next
        }

        // ////////////////////////////////////////////////////////////////////////////////
        // This "Get Next" Function Is For A* and Sets Up THe Costs Of The Search Node
        // ////////////////////////////////////////////////////////////////////////////////
        pub fn get_next_astar(
            &mut self,
            suser: &dyn std::any::Any,
            edge_parent_to_next: &TEDGE,
        ) -> &mut search_node {
            // assert(mNodesPtr);
            // assert(mNextIndex!=NULL_NODE_INDEX);
            //
            // //NOTE: we do not do a "mClosed.set_bit(mNextIndex);" here because A* only closes nodes that are visited
            //
            // mNext.mNode				= mNextIndex;
            // mNext.mCostToGoal		= suser.cost((*mNodesPtr)[mNext.mNode], (*mNodesPtr)[mEnd]);
            // mNext.mCostFromStart	= suser.cost(edge_parent_to_next, (*mNodesPtr)[mNext.mNode]);
            //
            // if (mNext.mParentVisit!=NULL_VISIT_INDEX)
            // {
            //     mNext.mCostFromStart += mVisited[mNext.mParentVisit].mCostFromStart;
            // }
            // return mNext;
            &mut self.m_next
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Setup The Search Data
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn setup_search(&self, sdata: &mut search) {
        // sdata.setup(&mNodes);
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // A* Search
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn astar(&self, sdata: &mut search, suser: &dyn std::any::Any) {
        // // Make Sure The Nodes We Are Searching For Exist
        // //------------------------------------------------
        // assert(MAXEDGES>1);
        // sdata.setup(&mNodes);
        //
        // // Allocate Our Data Structures
        // //------------------------------
        // handle_heap<search_node>		open(mNodes);
        // int								curNeighbor;
        // int								curEdge;
        // float							curCost;
        //
        // // Run Through The Open List
        // //---------------------------
        // open.push(sdata.get_next());
        // while (!open.empty() && !sdata.success())
        // {
        //     sdata.visit(open.top());
        //     open.pop();
        //
        //     // Search Through The Non Closed Nodes Edges
        //     //-------------------------------------------
        //     TNodeNeighbors&		curNeighbors = get_node_neighbors(sdata.mPrevIndex);
        //     for (curNeighbor=0; curNeighbor<curNeighbors.size(); curNeighbor++)
        //     {
        //         curEdge = curNeighbors[curNeighbor].mEdge;
        //         if (curEdge==-1 || suser.is_valid(mEdges[curEdge], sdata.mEnd))
        //         {
        //             sdata.mNextIndex			= curNeighbors[curNeighbor].mNode;
        //             search_node& snode			= sdata.get_next(suser, mEdges[curEdge]);
        //             curCost						= snode.cost_estimate();
        //
        //             // Is It Already In The Open List?
        //             //---------------------------------
        //             if (open.used(snode.mNode))
        //             {
        //                 if (curCost<(open[snode.mNode]).cost_estimate())
        //                 {
        //                     open[snode.mNode]		= snode;		// Use This As The Node (With New Parent & Cost)
        //                     open.reheapify(snode.mNode);			// Resort the node in the heap
        //                 }
        //             }
        //
        //             // Is It Already In The Closed List?
        //             //-----------------------------------
        //             else if (sdata.next_index_closed())
        //             {
        //                 if (curCost < sdata.visited_cost(snode.mNode))
        //                 {
        //                     sdata.reopen_next_index();				// Pull it off the closed list
        //                     open.push(snode);						// Add it to open
        //                 }
        //             }
        //
        //             // It Must Be A Whole New Node
        //             //------------------------------
        //             else
        //             {
        //                 open.push(snode);
        //             }
        //         }
        //     }
        // }
        //
        //
        // #if !defined(FINAL_BUILD)
        //     mSearchCount++;
        //     mSearchMemorySize += (sizeof(sdata) + sizeof(suser) + sizeof(open));
        //
        //     if (sdata.success())
        //     {
        //         mSearchSuccess++;
        //         mSearchSuccessVisited += sdata.num_visited();
        //         for (sdata.path_begin(); !sdata.path_end(); sdata.path_inc())
        //         {
        //             mSearchSuccessPathLen ++;
        //         }
        //     }
        //     else
        //     {
        //         mSearchFail++;
        //         mSearchFailVisited += sdata.num_visited();
        //     }
        // #endif
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Breadth First Search (Use Queue Open List)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn bfs(&self, sdata: &mut search) {
        // sdata.setup(&mNodes);
        //
        // // Allocate Our Data Structures
        // //------------------------------
        // ratl::queue_vs<search_node, MAXNODES>	open;
        //
        // // Run Through The Open List
        // //---------------------------
        // open.push(sdata.get_next());
        // while (open.size()>0 && !sdata.success())
        // {
        //     sdata.visit(open.top());
        //     open.pop();
        //
        //
        //     // Search Through The Non Closed Nodes Edges
        //     //-------------------------------------------
        //     for (sdata.mNextIndex=0; sdata.mNextIndex<MAXNODES; sdata.mNextIndex++)
        //     {
        //         if (!sdata.next_index_closed())
        //         {
        //             if (get_edge_across(sdata.mPrevIndex, sdata.mNextIndex))
        //             {
        //                 open.push(sdata.get_next());
        //             }
        //         }
        //     }
        // }
        //
        // #if !defined(FINAL_BUILD)
        //     mSearchCount++;
        //     mSearchMemorySize += (sizeof(sdata) + sizeof(open));
        //
        //     if (sdata.success())
        //     {
        //         mSearchSuccess++;
        //         mSearchSuccessVisited += sdata.num_visited();
        //     }
        //     else
        //     {
        //         mSearchFail++;
        //         mSearchFailVisited += sdata.num_visited();
        //     }
        // #endif
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Depth First Search (Use Stack Open List)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn dfs(&self, sdata: &mut search) {
        // sdata.setup(&mNodes);
        //
        // // Allocate Our Data Structures
        // //------------------------------
        // ratl::stack_vs<search_node, MAXNODES>	open;
        //
        // // Run Through The Open List
        // //---------------------------
        // open.push(sdata.get_next());
        // while (open.size()>0 && !sdata.success())
        // {
        //     sdata.visit(open.top());
        //     open.pop();
        //
        //
        //     // Search Through The Non Closed Nodes Edges
        //     //-------------------------------------------
        //     for (sdata.mNextIndex=0; sdata.mNextIndex<MAXNODES; sdata.mNextIndex++)
        //     {
        //         if (!sdata.next_index_closed())
        //         {
        //             if (get_edge_across(sdata.mPrevIndex, sdata.mNextIndex))
        //             {
        //                 open.push(sdata.get_next());
        //             }
        //         }
        //     }
        // }
        //
        // #if !defined(FINAL_BUILD)
        //     mSearchCount++;
        //     mSearchMemorySize += (sizeof(sdata) + sizeof(open));
        //
        //     if (sdata.success())
        //     {
        //         mSearchSuccess++;
        //         mSearchSuccessVisited += sdata.num_visited();
        //     }
        //     else
        //     {
        //         mSearchFail++;
        //         mSearchFailVisited += sdata.num_visited();
        //     }
        // #endif
    }

    // #if !defined(FINAL_BUILD)
    //     int		mSearchCount;
    //     int		mSearchMemorySize;
    //
    //     int		mSearchSuccess;
    //     int		mSearchSuccessVisited;
    //     int		mSearchSuccessPathLen;
    //
    //     int		mSearchFail;
    //     int		mSearchFailVisited;
    //
    //     fn ProfileSpew() {
    //         ProfilePrint("");
    //         ProfilePrint("");
    //         ProfilePrint("--------------------------------------------------------");
    //         ProfilePrint("RAVEN STANDARD LIBRARY  -  COMPUTATIONAL GEOMETRY MODULE");
    //         ProfilePrint("               Graph Profile Results                    ");
    //         ProfilePrint("--------------------------------------------------------");
    //         ProfilePrint("");
    //         ProfilePrint("GRAPH SIZE (Bytes): (%d)  (KiloBytes): (%5.3f)  MeggaBytes(%3.3f)",
    //             (sizeof(*this)),
    //             ((float)(sizeof(*this))/1024.0f),
    //             ((float)(sizeof(*this))/1048576.0f)
    //             );
    //         ProfilePrint("GRAPH COUNT: (%d) Nodes  (%d) Edges", mNodes.size(), mEdges.size());
    //         if (mNodes.size())
    //         {
    //             ProfilePrint("GRAPH COUNT: (%f) Edges/Node", (float)mEdges.size()/(float)mNodes.size());
    //         }
    //         ProfilePrint("");
    //         if (mSearchCount)
    //         {
    //             ProfilePrint("SEARCH: (%d) Searches  (%f) AveSize", mSearchCount,  ((float)mSearchMemorySize/(float)mSearchCount));
    //             if (mSearchSuccess)
    //             {
    //                 ProfilePrint("SEARCH: (%d) Successes  (%f) AveVisited  (%f) AvePathLen",
    //                     mSearchSuccess,
    //                     ((float)mSearchSuccessVisited/(float)mSearchSuccess),
    //                     ((float)mSearchSuccessPathLen/(float)mSearchSuccess)
    //                     );
    //             }
    //             if (mSearchFail)
    //             {
    //                 ProfilePrint("SEARCH: (%d) Failures  (%f) AveVisited", mSearchFail,  ((float)mSearchFailVisited/(float)mSearchFail));
    //             }
    //         }
    //         ProfilePrint("");
    //     }
    // #endif
}
