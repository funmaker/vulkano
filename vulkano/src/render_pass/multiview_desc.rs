// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

/// The description of a multiview of a render pass.
#[derive(Clone, Debug)]
pub struct MultiviewDesc {
    view_masks: Vec<u32>,
    view_offsets: Vec<i32>,
    correlation_masks: Vec<u32>,
}

impl MultiviewDesc {
    /// Creates a description of a multipass.
    pub fn new(
        view_masks: Vec<u32>,
        view_offsets: Vec<i32>,
        correlation_masks: Vec<u32>,
    ) -> MultiviewDesc {
        MultiviewDesc {
            view_masks,
            view_offsets,
            correlation_masks,
        }
    }
    
    /// Creates an empty description that doesn't enable multiview functionality.
    pub fn empty() -> MultiviewDesc {
        MultiviewDesc {
            view_masks: vec![],
            view_offsets: vec![],
            correlation_masks: vec![],
        }
    }

    /// Creates a description of single subpass multiview with given number of views with no offets
    /// and no correlation masks.
    ///
    /// # Panic
    ///
    /// - Panics if `views` is greater than 32.
    pub fn with_views(views: u8) -> MultiviewDesc {
        debug_assert!(views <= 32);

        let mask = ((1_u64 << views) - 1) as u32;
    
        MultiviewDesc {
            view_masks: vec![mask],
            view_offsets: vec![],
            correlation_masks: vec![],
        }
    }
    
    /// Creates a description of single subpass multiview with given number of correlated views.
    ///
    /// # Panic
    ///
    /// - Panics if `views` is greater than 32.
    pub fn with_views_correlated(views: u8) -> MultiviewDesc {
        debug_assert!(views <= 32);
        
        let mask = ((1_u64 << views) - 1) as u32;
        
        MultiviewDesc {
            view_masks: vec![mask],
            view_offsets: vec![],
            correlation_masks: vec![mask],
        }
    }

    // Returns the view masks of the description.
    #[inline]
    pub fn view_masks(&self) -> &[u32] {
        &self.view_masks
    }

    // Returns the view offsets of the description.
    #[inline]
    pub fn view_offsets(&self) -> &[i32] {
        &self.view_offsets
    }

    // Returns the correlation masks of the description.
    #[inline]
    pub fn correlation_masks(&self) -> &[u32] {
        &self.correlation_masks
    }
}

impl Default for MultiviewDesc {
    fn default() -> Self {
        Self::empty()
    }
}
