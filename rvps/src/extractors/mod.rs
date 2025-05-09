// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

//! Extractors for RVPS.

pub mod extractor_modules;

use anyhow::*;
use std::collections::HashMap;

use self::extractor_modules::{ExtractorInstance, ExtractorModuleList};
use super::{Message, ReferenceValue};

#[derive(Default)]
pub struct Extractors {
    /// A map of provenance types to Extractor initializers
    extractors_module_list: ExtractorModuleList,

    /// A map of provenance types to Extractor instances
    extractors_instance_map: HashMap<String, ExtractorInstance>,
}

impl Extractors {
    /// Register an `Extractor` instance to `Extractors`. The `Extractor` is responsible for
    /// handling specific kind of provenance (as `extractor_name` indicates).
    fn register_instance(&mut self, extractor_name: String, extractor_instance: ExtractorInstance) {
        self.extractors_instance_map
            .insert(extractor_name, extractor_instance);
    }

    /// Instantiate an `Extractor` of given type `extractor_name`. This method will
    /// instantiate an `Extractor` instance and then register it.
    fn instantiate_extractor(&mut self, extractor_name: String) -> Result<()> {
        let instantiate_func = self.extractors_module_list.get_func(&extractor_name)?;
        let extractor_instance = (instantiate_func)();
        self.register_instance(extractor_name, extractor_instance);
        Ok(())
    }

    /// Process the message, by verifying the provenance
    /// and extracting reference values within.
    /// If provenance is valid, return all of the relevant
    /// reference values.
    /// Each ReferenceValue digest is expected to be base64 encoded.
    pub fn process(&mut self, message: Message) -> Result<Vec<ReferenceValue>> {
        let typ = message.r#type;

        if self.extractors_instance_map.get_mut(&typ).is_none() {
            self.instantiate_extractor(typ.clone())?;
        }
        let extractor_instance = self
            .extractors_instance_map
            .get_mut(&typ)
            .ok_or_else(|| anyhow!("The Extractor instance does not existing!"))?;

        extractor_instance.verify_and_extract(&message.payload)
    }
}
