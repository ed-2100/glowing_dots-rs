use anyhow::{Context, Ok, Result, anyhow};
use log::info;
use roxmltree::{Document, Node, NodeType};
use std::collections::BTreeMap;

fn main() -> Result<()> {
    env_logger::init();

    let doc = Parser::parse(include_str!("../vk.xml"))?;

    println!("Here's what we've got so far:\n{:#?}", doc);

    info!("{}", i64::from_str_radix("555", 16)?);

    Ok(())
}

#[derive(Debug)]
struct Tag {
    author: String,
    contact: String,
}

#[derive(Debug)]
struct Enum {
    bitwidth: usize,
    ty: EnumType,
    fields: BTreeMap<String, EnumField>,
}

#[derive(Debug)]
enum EnumType {
    Enum,
    Bitmask
}

#[derive(Debug)]
enum EnumField {
    Regular {
        value: String,
        deprecated: bool,
        comment: Option<String>,
    },
    Alias {
        alias: String,
        deprecated: bool,
    },
}

#[derive(Debug)]
struct Constant {
    ty: String,
    value: String,
    comment: Option<String>,
}

#[derive(Default, Debug)]
struct Parser {
    tags: BTreeMap<String, Tag>,
    constants: BTreeMap<String, Constant>,
    enums: BTreeMap<String, Enum>,
}

// Public functions.
impl Parser {
    pub fn parse(data: &str) -> Result<Parser> {
        let doc = Document::parse(data)?;

        let mut registry = Parser::default();

        let root = doc.root();

        registry.parse_root(&root)?;

        Ok(registry)
    }
}

// Parse functions.
impl Parser {
    fn parse_root(&mut self, node: &Node) -> Result<()> {
        for c in node.children() {
            let ty = c.node_type();
            match ty {
                NodeType::Root | NodeType::PI => Err(anyhow!("Unexpected node type: {:?}", ty))?,
                NodeType::Element => {}
                NodeType::Comment | NodeType::Text => continue,
            }

            let name = c.tag_name().name();
            match name {
                "registry" => self.parse_registry(&c),
                _ => Err(anyhow!("0 Unexpected node type: {}.", name)),
            }?
        }

        Ok(())
    }

    fn parse_registry(&mut self, node: &Node) -> Result<()> {
        for c in node.children().filter(|node| node.is_element()) {
            let name = c.tag_name().name();

            match name {
                "comment" => continue,
                "platforms" => {}
                "tags" => self.parse_tags(&c)?,
                "types" => {}
                "enums" => self.parse_enums(&c)?,
                "commands" => {}
                "feature" => {}
                "extensions" => {}
                "formats" => {}
                "spirvextensions" => {}
                "spirvcapabilities" => {}
                "sync" => {}
                "videocodecs" => {}
                _ => Err(anyhow!("Unexpected node tag: {}.", name))?,
            }
        }

        Ok(())
    }

    fn parse_tags(&mut self, node: &Node) -> Result<()> {
        for c in node.children().filter(|node| node.is_element()) {
            let name = c.tag_name().name();
            match name {
                "comment" => continue,
                "tag" => {
                    let tag_name = c
                        .attribute("name")
                        .with_context(|| "name is required for tag")?;

                    if self
                        .tags
                        .insert(
                            tag_name.to_string(),
                            Tag {
                                author: c
                                    .attribute("author")
                                    .with_context(|| "author is required for tag")?
                                    .to_string(),
                                contact: c
                                    .attribute("contact")
                                    .with_context(|| "contact is required for tag")?
                                    .to_string(),
                            },
                        )
                        .is_some()
                    {
                        Err(anyhow!("{} already exists in the map.", tag_name))?
                    }
                }
                _ => Err(anyhow!("Expected tag \"tag\", but got \"{}\".", name))?,
            }
        }

        Ok(())
    }

    fn parse_enums(&mut self, node: &Node) -> Result<()> {
        let ty = node
            .attribute("type")
            .with_context(|| "type is required for enums")?;
        let bitwidth = node.attribute("bitwidth").unwrap_or("32").parse()?;
        match bitwidth {
            8 | 16 | 32 | 64 | 128 => {}
            _ => Err(anyhow!("Invalid bitwidth."))?,
        }

        match ty {
            "constants" => self.parse_enums_constants(node)?,
            "enum" => self.parse_enums_enum(node, EnumType::Enum, bitwidth)?,
            "bitmask" => self.parse_enums_enum(node, EnumType::Bitmask, bitwidth)?,
            _ => Err(anyhow!("Unexpected enum type \"{}\".", ty))?,
        }

        Ok(())
    }

    fn parse_enums_constants(&mut self, node: &Node) -> Result<()> {
        for c in node.children().filter(|node| node.is_element()) {
            let name = c.tag_name().name();
            match name {
                "comment" | "unused" => continue,
                "enum" => {
                    if let Some(api) = c.attribute("api") {
                        if !api.split(",").map(|s| s.trim()).any(|s| s == "vulkan") {
                            continue;
                        }
                    }

                    let (k, v) = Self::parse_enums_constants_constant(&c)?;
                    if self.constants.insert(k.to_string(), v).is_some() {
                        Err(anyhow!("{} already exists in the map.", k))?
                    }
                }
                _ => Err(anyhow!("Expected tag \"enum\", but got \"{}\".", name))?,
            }
        }

        Ok(())
    }

    fn parse_enums_enum(&mut self, node: &Node, ty:EnumType, bitwidth: usize) -> Result<()> {
        let mut fields = BTreeMap::new();

        for c in node.children().filter(|node| node.is_element()) {
            let tag_name = c.tag_name().name();
            match tag_name {
                "comment" | "unused" => continue,
                "enum" => {
                    if let Some(api) = c.attribute("api") {
                        if !api.split(",").map(|s| s.trim()).any(|s| s == "vulkan") {
                            continue;
                        }
                    }

                    let (k, v) = Self::parse_enums_enum_field(&c)?;
                    if fields.insert(k.to_string(), v).is_some() {
                        Err(anyhow!("{} aready exists in the map.", k))?
                    }
                }
                _ => Err(anyhow!("Expected tag \"enum\", but got \"{}\".", tag_name))?,
            }
        }

        let name = node
            .attribute("name")
            .with_context(|| "name is required for enum")?;

        if self
            .enums
            .insert(name.to_string(), Enum { fields, ty, bitwidth })
            .is_some()
        {
            Err(anyhow!("{} already exists in the map.", name))?
        }

        Ok(())
    }

    fn parse_enums_constants_constant<'a>(c: &'a Node<'a, 'a>) -> Result<(&'a str, Constant)> {
        let field_name = c
            .attribute("name")
            .with_context(|| "name is required for constant")?;

        let field_data = Constant {
            ty: c
                .attribute("type")
                .with_context(|| "type is required for constant")?
                .to_string(),
            value: c
                .attribute("value")
                .with_context(|| "value is required for constant")?
                .to_string(),
            comment: c.attribute("comment").map(|s| s.to_string()),
        };

        Ok((field_name, field_data))
    }

    fn parse_enums_enum_field<'a>(c: &'a Node<'a, 'a>) -> Result<(&'a str, EnumField)> {
        let value = c.attribute("value");
        let bitpos = c.attribute("bitpos");
        let deprecated = c.attribute("deprecated").is_some();

        let field_name = c
            .attribute("name")
            .with_context(|| "name is required for enum field")?;

        let field_data = if value.is_some() && bitpos.is_some() {
            Err(anyhow!("Can't have bitpos and value at the same time."))?
        } else if let Some(t) = value {
            EnumField::Regular {
                value: t.to_string(),
                deprecated,
                comment: c.attribute("comment").map(|s| s.to_string()),
            }
        } else if let Some(t) = bitpos {
            EnumField::Regular {
                value: format!("1 << {}", t.parse::<usize>()?),
                deprecated,
                comment: c.attribute("comment").map(|s| s.to_string()),
            }
        } else {
            EnumField::Alias {
                alias: c
                    .attribute("alias")
                    .with_context(|| "Expected alias for bitmask.")?
                    .to_string(),
                deprecated,
            }
        };

        Ok((field_name, field_data))
    }
}
