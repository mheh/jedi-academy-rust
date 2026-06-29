#![allow(non_snake_case)]

////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Graph Region
// ------------
//
//
////////////////////////////////////////////////////////////////////////////////////////

// Includes
// Local stubs for external dependencies (ragl_common.h, graph_vs.h)
// These would normally be defined in separate modules, but we declare them here
// for structural coherence of this file per the porting style guide.

pub mod ratl {
    use core::marker::PhantomData;

    /// Stub for ratl_base - empty base class equivalent
    pub struct ratl_base;

    /// Stub for vector_vs - dynamic array equivalent
    pub struct vector_vs<T, const N: usize> {
        _phantom: PhantomData<T>,
    }

    impl<T, const N: usize> Default for vector_vs<T, N> {
        fn default() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    /// Stub for pool_vs - pool allocator equivalent
    pub struct pool_vs<T, const N: usize> {
        _phantom: PhantomData<T>,
    }

    impl<T, const N: usize> Default for pool_vs<T, N> {
        fn default() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    /// Stub for grid2_vs - 2D grid equivalent
    pub struct grid2_vs<T, const ROWS: usize, const COLS: usize> {
        _phantom: PhantomData<T>,
    }

    impl<T, const ROWS: usize, const COLS: usize> Default for grid2_vs<T, ROWS, COLS> {
        fn default() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    /// Stub for bits_vs - bitfield equivalent
    pub struct bits_vs<const N: usize>;

    impl<const N: usize> Default for bits_vs<N> {
        fn default() -> Self {
            bits_vs
        }
    }
}

pub mod ragl {
    use super::ratl;
    use core::ffi::{c_int, c_short};
    use core::marker::PhantomData;

    /// Stub for graph_vs template
    pub struct graph_vs<TNODE, const MAXNODES: usize, TEDGE, const MAXEDGES: usize, const NUM_EDGES_PER_NODE: usize> {
        _phantom_node: PhantomData<TNODE>,
        _phantom_edge: PhantomData<TEDGE>,
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // The Graph Region Class
    ////////////////////////////////////////////////////////////////////////////////////////
    pub struct graph_region<
        TNODE,
        const MAXNODES: usize,
        TEDGE,
        const MAXEDGES: usize,
        const NUM_EDGES_PER_NODE: usize,
        const MAXREGIONS: usize,
        const MAXREGIONEDGES: usize,
    > {
        // Type aliases (from original template typedef):
        // TGraph = ragl::graph_vs<TNODE, MAXNODES, TEDGE, MAXEDGES, NUM_EDGES_PER_NODE>
        // TRegions = ratl::vector_vs<int, MAXNODES>
        // TRegionEdge = ratl::vector_vs<short, MAXREGIONS>
        // TEdges = ratl::pool_vs<TRegionEdge, MAXREGIONEDGES>
        // TLinks = ratl::grid2_vs<short, MAXREGIONS, MAXREGIONS>
        // TClosed = ratl::bits_vs<MAXREGIONS>

        mGraph: *const graph_vs<TNODE, MAXNODES, TEDGE, MAXEDGES, NUM_EDGES_PER_NODE>,
        mRegions: ratl::vector_vs<c_int, MAXNODES>,
        mRegionCount: c_int,
        mReservedRegionCount: c_int,
        mLinks: ratl::grid2_vs<c_short, MAXREGIONS, MAXREGIONS>,
        mEdges: ratl::pool_vs<ratl::vector_vs<c_short, MAXREGIONS>, MAXREGIONEDGES>,
        mClosed: ratl::bits_vs<MAXREGIONS>,
    }

    impl<
        TNODE,
        const MAXNODES: usize,
        TEDGE,
        const MAXEDGES: usize,
        const NUM_EDGES_PER_NODE: usize,
        const MAXREGIONS: usize,
        const MAXREGIONEDGES: usize,
    > graph_region<TNODE, MAXNODES, TEDGE, MAXEDGES, NUM_EDGES_PER_NODE, MAXREGIONS, MAXREGIONEDGES>
    {
        ////////////////////////////////////////////////////////////////////////////////////
        // Capacity Enum
        ////////////////////////////////////////////////////////////////////////////////////
        const NULL_REGION: c_int = -1;
        const NULL_EDGE: c_int = -1;
        const CAPACITY: usize = MAXREGIONS;

        ////////////////////////////////////////////////////////////////////////////////////
        // Constructor
        ////////////////////////////////////////////////////////////////////////////////////
        pub fn new(graph: *const graph_vs<TNODE, MAXNODES, TEDGE, MAXEDGES, NUM_EDGES_PER_NODE>) -> Self {
            let mut result = Self {
                mGraph: graph,
                mRegions: Default::default(),
                mRegionCount: 0,
                mReservedRegionCount: 0,
                mLinks: Default::default(),
                mEdges: Default::default(),
                mClosed: Default::default(),
            };
            result.clear();
            result
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // Clear Out All Temp Data So We Can Recalculate Regions
        ////////////////////////////////////////////////////////////////////////////////////
        pub fn clear(&mut self) {
            // Note: These operations would resize/initialize the collections
            // mRegions.resize(0, (int)NULL_REGION);
            // mRegions.resize(MAXNODES, (int)NULL_REGION);
            self.mRegionCount = 0;
            self.mReservedRegionCount = 0;

            // mLinks.init(NULL_EDGE);

            // for (int i=0; i<MAXREGIONEDGES; i++)
            // {
            //     if (mEdges.is_used(i))
            //     {
            //         mEdges[i].resize(0, NULL_EDGE);
            //     }
            // }
            // mEdges.clear();
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // How Many Regions Have Been Created
        ////////////////////////////////////////////////////////////////////////////////////
        pub fn size(&self) -> c_int {
            self.mRegionCount
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // Get The Region For A Given Node
        ////////////////////////////////////////////////////////////////////////////////////
        pub fn get_node_region(&self, node: c_int) -> c_int {
            // return mRegions[mGraph.node_index(Node)];
            // Placeholder: actual implementation would need access to mGraph methods
            0
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // Call this function to find out if it is at all possible to get from nodeA to
        // nodeB.  If there is no possible connection, or there is one, but the connection
        // is not valid at the current time, this routine will return false.  Use it as
        // a quick cull routine before a search.
        //
        // In order to use this function, you must have an EdgeQuery class (use the default
        // above, or derive your own for more specialized behavior).
        ////////////////////////////////////////////////////////////////////////////////////
        pub fn has_valid_edge<U>(&mut self, node_a: c_int, node_b: c_int, user: &U) -> bool {
            // int RegionA = mRegions[NodeA];
            // int RegionB = mRegions[NodeB];

            let region_a = 0; // Placeholder
            let region_b = 0; // Placeholder

            if region_a == region_b {
                return true;
            }

            // mClosed.clear();

            self.has_valid_region_edge(region_a, region_b, user)
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // Reserve Region
        //
        // Allows a user to pre-allocate a special region for a group of points
        ////////////////////////////////////////////////////////////////////////////////////
        pub fn reserve(&mut self) -> c_int {
            assert!(self.mRegionCount < (MAXREGIONS as c_int - 1));
            if self.mRegionCount >= (MAXREGIONS as c_int - 1) {
                // stop adding points, we're full, you MUST increase MAXREGIONS for this to work
                return Self::NULL_REGION;
            }
            self.mReservedRegionCount += 1;
            self.mRegionCount += 1;
            self.mRegionCount
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // assign_region
        //
        // Allows a user to pre-allocate a special region for a group of points
        ////////////////////////////////////////////////////////////////////////////////////
        pub fn assign_region(&mut self, node_index: c_int, region_index: c_int) {
            // mRegions[NodeIndex] = RegionIndex;
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // Define Regions
        //
        // Scan through all the nodes (calling the depth first recursive traversal below),
        // and mark regions of nodes which can traverse to one another without needing to check
        // for valid edges.
        //
        ////////////////////////////////////////////////////////////////////////////////////
        pub fn find_regions<U>(&mut self, user: &U) -> bool {
            // for (TGraph::TNodes::iterator i=mGraph.nodes_begin(); i!=mGraph.nodes_end(); i++)
            // {
            //     CurNodeIndex = i.index();
            //     if (mRegions[CurNodeIndex] == NULL_REGION)
            //     {
            //         assert(mRegionCount < (MAXREGIONS-1));
            //         if (mRegionCount >= (MAXREGIONS-1) )
            //         {//stop adding points, we're full, you MUST increase MAXREGIONS for this to work
            //             return false;
            //         }
            //         mRegionCount ++;				// Allocate The New Region
            //         assign(CurNodeIndex, user);		// Assign All Points To It
            //     }
            // }
            // mRegionCount ++;		// Size is actually 1 greater than the number of regions
            true
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // Search For All Possible Edges Which Connect Regions
        //
        // Once called, this class will have reference data for how to get from one region
        // to another.
        ////////////////////////////////////////////////////////////////////////////////////
        pub fn find_region_edges(&mut self) -> bool {
            let mut success = true;

            // for (int indexA=0; indexA<MAXNODES; indexA++)
            // {
            //     RegionA = mRegions[indexA];
            //     if (RegionA!=NULL_REGION)
            //     {
            //         for (int indexB=0; indexB<MAXNODES; indexB++)
            //         {
            //             RegionB				= mRegions[indexB];
            //             ReservedRegionLink	= (RegionA<=mReservedRegionCount || RegionB<=mReservedRegionCount);
            //             if (RegionB!=NULL_REGION && RegionB!=RegionA && mGraph.get_edge_across(indexA, indexB))
            //             {
            //                 RegionLink = mLinks.get(RegionA, RegionB);
            //
            //                 // Do We Need To Allocate A New Region Link Vector?
            //                 //--------------------------------------------------
            //                 if (RegionLink==-1)
            //                 {
            //                     if (ReservedRegionLink)
            //                     {
            //                         mLinks.get(RegionA, RegionB) = -2;		// Special Flag For Reserved Regions - they have no edges
            //                         mLinks.get(RegionB, RegionA) = -2;
            //                     }
            //                     else
            //                     {
            //                         if (mEdges.full())
            //                         {
            //                             assert("graph_region: Too Many Region Edges"==0);
            //                             Success = false;
            //                         }
            //                         else
            //                         {
            //                             RegionLink = mEdges.alloc();
            //                             mEdges[RegionLink].resize(0, NULL_EDGE);
            //                             mEdges[RegionLink].push_back(mGraph.get_edge_across(indexA, indexB));
            //
            //                             mLinks.get(RegionA, RegionB) = RegionLink;
            //                             mLinks.get(RegionB, RegionA) = RegionLink;
            //                         }
            //                     }
            //                 }
            //
            //
            //                 // Add This Edge To The Other Region Links
            //                 //-----------------------------------------
            //                 else if (!ReservedRegionLink)
            //                 {
            //                     mEdges[RegionLink].push_back(mGraph.get_edge_across(indexA, indexB));
            //                 }
            //             }
            //         }
            //     }
            // }

            success
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // This Routine Is A Depth First Recursive Traversal
        //
        // It will visit all neighbors for each node which have not already been visited
        // and assigned to a region.  Neighbors must always be valid.
        ////////////////////////////////////////////////////////////////////////////////////
        fn assign<U>(&mut self, node: c_int, user: &U) {
            // mRegions[Node] = mRegionCount;
            // for (int i=0; i<MAXNODES; i++)
            // {
            //     if (mRegions[i]==-1)
            //     {
            //         int edgeNum = mGraph.get_edge_across(Node, i);
            //         if (edgeNum && !user.can_be_invalid(mGraph.get_edge(edgeNum)))
            //         {
            //             assign(i, user);
            //         }
            //     }
            // }
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // This Routine Is A Depth First Recursive Search For Target Region
        //
        // Visited regions are makred on the "closed" bit field.
        ////////////////////////////////////////////////////////////////////////////////////
        fn has_valid_region_edge<U>(&mut self, cur_region: c_int, target_region: c_int, user: &U) -> bool {
            // Mark The Cur Region As Visited, So We Don't Try To Return To It
            //-----------------------------------------------------------------
            // mClosed.set_bit(CurRegion);

            // If The Two Nodes Are In The Same Region, Then This Is Valid
            //-------------------------------------------------------------
            if cur_region == target_region {
                return true;
            }

            // Scan Through The Cur Region's Neighbors With Currently Valid Region Edges
            //---------------------------------------------------------------------------
            // for (int NextRegion=0; NextRegion<mRegionCount; NextRegion++)
            // {
            //     // Check If The Link Exists And We Have Not Already Visited The Next Region
            //     //--------------------------------------------------------------------------
            //     CurRegionEdge = mLinks.get(CurRegion, NextRegion);
            //     if (CurRegionEdge!=NULL_EDGE && !mClosed.get_bit(NextRegion))
            //     {
            //         if (CurRegion<=mReservedRegionCount)
            //         {
            //             // Great, So We Have Found A Valid Neighboring Region, Search There
            //             //------------------------------------------------------------------
            //             if (has_valid_region_edge(NextRegion, TargetRegion, user))
            //             {
            //                 return true;		// HEY!  Somehow, Going To Next Region Got Us To The Target Region!
            //             }
            //         }
            //         else
            //         {
            //             // Scan Through This Region Edge List Of Graph Edges For Any Valid One
            //             //---------------------------------------------------------------------
            //             assert(mEdges[CurRegionEdge].size()>0);
            //             for (int j=0; j<mEdges[CurRegionEdge].size(); j++)
            //             {
            //                 if (user.is_valid(
            //                                 mGraph.get_edge(mEdges[CurRegionEdge][j]),
            //                                 (NextRegion==TargetRegion)?(-1):(0)
            //                                 )
            //                     )
            //                 {
            //                     // Great, So We Have Found A Valid Neighboring Region, Search There
            //                     //------------------------------------------------------------------
            //                     if (has_valid_region_edge(NextRegion, TargetRegion, user))
            //                     {
            //                         return true;		// HEY!  Somehow, Going To Next Region Got Us To The Target Region!
            //                     }
            //
            //                     // Ok, The Target Region Turned Out To Be A Dead End, We Can Stop Trying To Get There
            //                     //------------------------------------------------------------------------------------
            //                     break;
            //                 }
            //             }
            //         }
            //     }
            // }

            // Nope, We Failed To Find Any Valid Region Edges Which Lead To The Target Region
            //--------------------------------------------------------------------------------
            false
        }

        #[cfg(not(feature = "FINAL_BUILD"))]
        pub fn profile_spew(&self) {
            // ProfilePrint("");
            // ProfilePrint("");
            // ProfilePrint("--------------------------------------------------------");
            // ProfilePrint("RAVEN STANDARD LIBRARY  -  COMPUTATIONAL GEOMETRY MODULE");
            // ProfilePrint("              Region Profile Results                    ");
            // ProfilePrint("--------------------------------------------------------");
            // ProfilePrint("");
            // ProfilePrint("REGION SIZE (Bytes): (%d)  (KiloBytes): (%5.3f)", sizeof(*this), ((float)(sizeof(*this))/1024.0f));
            // ProfilePrint("REGION COUNT: (%d) Regions  (%d) Edges", mRegionCount, mEdges.size());
            // if (mRegionCount)
            // {
            //     int RegionEdges = 0;
            //     for (TEdges::iterator it=mEdges.begin(); it!=mEdges.end(); it++)
            //     {
            //         RegionEdges += (*it).size();
            //     }
            //     ProfilePrint("REGION COUNT: (%f) Ave Edges Size", (float)RegionEdges / (float)mRegionCount);
            // }
            // ProfilePrint("");
        }
    }
}
