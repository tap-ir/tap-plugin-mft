//! MBR plugin parse an MFT and extract file and attribute into restruct tree

use tap::config_schema;
use tap::plugin;
use tap::plugin::{PluginInfo, PluginInstance, PluginConfig, PluginArgument, PluginResult, PluginEnvironment};
use tap::tree::{TreeNodeId, TreeNodeIdSchema};
use tap::node::Node;
use tap::error::RustructError;

use anyhow::Result;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

use tap_plugin_ntfs::ntfs::Ntfs;
use log::warn;

plugin!("mft", "File system", "Read and parse MFT file", MftPlugin, Arguments);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Arguments
{
  #[schemars(with = "TreeNodeIdSchema")] 
  file : TreeNodeId,
  sector_size : Option<u16>,
  mft_record_size : Option<u32>,
}

#[derive(Debug, Serialize, Deserialize,Default)]
pub struct Results
{
}

#[derive(Default)]
pub struct MftPlugin
{
}

impl MftPlugin
{
  fn run(&mut self, args : Arguments, env : PluginEnvironment) -> Result<Results>
  {
    let file_node = env.tree.get_node_from_id(args.file).ok_or(RustructError::ArgumentNotFound("file"))?;
    file_node.value().add_attribute(self.name(), None, None); 
    let value = file_node.value().get_value("data").ok_or(RustructError::ValueNotFound("data"))?;
    let master_mft = value.try_as_vfile_builder().ok_or(RustructError::ValueTypeMismatch)?;

    warn!("Running MFT plugin");

    let mut ntfs = Ntfs::from_mft(master_mft, args.sector_size, args.mft_record_size)?;
    ntfs.create_nodes(&env.tree);
    let ntfs_node = Node::new("ntfs");
    let ntfs_node_id = env.tree.add_child(args.file, ntfs_node)?;
    let orphan_node = Node::new("orphan");
    let orphan_node_id = env.tree.add_child(ntfs_node_id, orphan_node)?;
    ntfs.link_nodes(&env.tree, ntfs_node_id, orphan_node_id);

    Ok(Results{})
  }
}
