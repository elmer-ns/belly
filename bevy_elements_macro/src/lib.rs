use bevy_elements_core::attributes::component;
use proc_macro2::TokenStream;
use quote::*;
extern crate proc_macro;
use syn::{Expr, spanned::Spanned, Error, ExprPath};
use syn_rsx::{Node, NodeType, parse, NodeAttribute};

fn create_single_command_stmt(expr: &ExprPath) -> TokenStream {
    let component_span = expr.span();
    if let Some(component) = expr.path.get_ident() {
        if component.to_string().chars().next().unwrap().is_uppercase() {
            quote_spanned! {component_span=>
                c.insert(#component::default());
            }
        } else {
            quote_spanned! {component_span=>
                c.insert(#component);
            }
        }
    } else {
        Error::new(component_span, "Invalid components declaration").into_compile_error()
    }
}

fn create_command_stmts(expr: &Expr) -> TokenStream {
    let with_body = match expr {
        Expr::Path(path) => create_single_command_stmt(path),
        Expr::Tuple(components) => {
            let mut components_expr = quote! { };
            for component_expr in components.elems.iter() {
                let component_span = component_expr.span();
                if let Expr::Path(component) = component_expr {
                    let component_expr = create_single_command_stmt(component);
                    components_expr = quote_spanned! {component_span=>
                        #components_expr
                        #component_expr
                    };
                } else {
                    return Error::new(component_span, "Invalid component name").into_compile_error();
                }
            }
            components_expr
        },
        _ => {
            return Error::new(expr.span(), "Invalid components declaration").into_compile_error();
        }
    };
    let expr_span = expr.span();
    quote_spanned! {expr_span=>
        __ctx.attributes.add(::bevy_elements_core::attributes::Attribute::from_commands("with", ::std::boxed::Box::new(move |c| {
            #with_body
        })));
    }
}

fn create_attr_stmt(attr: &NodeAttribute) -> TokenStream {
    let attr_name = attr.key.to_string();
    match &attr.value {
        None => {
            return quote! {
                __ctx.attributes.add(::bevy_elements_core::attributes::Attribute::new(
                    #attr_name.into(),
                    ::bevy_elements_core::attributes::AttributeValue::Empty
                ));
            };
        }
        Some(attr_value) => {
            let attr_value = attr_value.as_ref();
            if attr_name == "with" {
                return create_command_stmts(attr_value);
            } else {
                return quote! {
                    __ctx.attributes.add(::bevy_elements_core::attributes::Attribute::new(
                        #attr_name.into(),
                        (#attr_value).into()
                    ));
                }
            }
        }
    }

}

fn walk_nodes<'a>(element: &'a Node, create_entity: bool) -> TokenStream {
    let mut children = quote! { };
    if let Node::Element(element) = element {
        for attr in element.attributes.iter() {
            if let Node::Attribute(attr) = attr {
                let attr_stmt = create_attr_stmt(attr);
                children = quote! {
                    #children
                    #attr_stmt
                };
            }
        }
        for child in element.children.iter() {
            match child {
                Node::Element(_) => {
                    let expr = walk_nodes(child, true);
                    children = quote! {
                        #children
                        __ctx.add_child( #expr );
                    };
                },
                Node::Text(text) => {
                    let text = text.value.as_ref();
                    children = quote! {
                        #children
                        {
                            let __text_entity = __world.spawn().id();
                            __ctx.add_child(__text_entity.clone());
                            ::bevy_elements_core::context::internal::push_text(__world, __text_entity, #text.to_string());
                            __world
                                .resource::<::bevy_elements_core::builders::TextElementBuilder>().clone()
                                .build(__world);
                            ::bevy_elements_core::context::internal::pop_context(__world);
                        }
                    };
                },
                Node::Block(block) => {
                    let block = block.value.as_ref();
                    let block_span = block.span();
                    children = quote_spanned! { block_span=>
                        #children
                        for __child in #block.into_content(__world).iter() {
                            __ctx.add_child( __child.clone() );
                        }
                    }
                }
                _ => ()
            };
        }
        let tag = element.name.to_string();
        let invalid_element_msg = format!("Invalid tag name: {}", tag);
        let parent = if create_entity {
            quote! { let __parent = __world.spawn().id(); }
        } else {
            quote! { }
        };
        quote! {
            {
                #parent
                let __tag_name = #tag.into();
                let mut __ctx = ::bevy_elements_core::context::ElementContext::new(__tag_name, __parent);
                
                #children
                
                ::bevy_elements_core::context::internal::push_element(__world, __ctx);
                __world
                    .resource::<::bevy_elements_core::builders::ElementBuilderRegistry>()
                    .get_builder(__tag_name)
                    .expect( #invalid_element_msg )
                    .build(__world);
                ::bevy_elements_core::context::internal::pop_context(__world);
                __parent
            }
        }
    } else {
        quote! { }
    }

}


#[proc_macro]
pub fn bsx(tree: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse(tree.into()) {
        Err(err) => err.to_compile_error().into(),
        Ok(nodes) => {
            let body = walk_nodes(&nodes[0], false);
            // nodes[0]
            let wraped = quote! {
                ::bevy_elements_core::ElementsBuilder::new(
                    move |
                        __world: &mut ::bevy::prelude::World,
                        __parent: ::bevy::prelude::Entity
                    | {
                        #body;
                    }
            )};
            proc_macro::TokenStream::from(wraped)
        }
    }
}


// pub fn components(tree: proc_macro::TokenStream) -> proc_macro::TokenStream {


// }


// #macro_rules!  {
//     () => {
        
//     };
// }