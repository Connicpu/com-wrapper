use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericArgument, Ident, Index, Member, Meta, NestedMeta,
    PathArguments, Type,
};

pub fn expand_com_wrapper(input: &DeriveInput) -> Result<TokenStream, String> {
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => return Err("ComWrapper can only wrap a `struct`".into()),
    };

    let (member, itype, ctype) = get_comptr_member(fields)?;
    let attrinfo = parse_attr(&input.attrs)?;

    let wrapper_impl = wrapper_impl(&input.ident, &member, itype, ctype);
    let meta_impl = meta_impl(&input.ident, &attrinfo);

    Ok(quote! {
        #wrapper_impl
        #meta_impl
    })
}

fn wrapper_impl(wrap: &Ident, member: &Member, itype: &Type, ctype: &Type) -> TokenStream {
    let ptr_wrap = create_wrapping(wrap, member);
    quote! {
        impl ::com_wrapper::ComWrapper for #wrap {
            type Interface = #itype;
            #[inline]
            unsafe fn get_raw(&self) -> *mut #itype {
                ComPtr::as_raw(&self.#member)
            }
            #[inline]
            unsafe fn from_raw(ptr: *mut #itype) -> Self {
                <Self as ::com_wrapper::ComWrapper>::from_ptr(ComPtr::from_raw(ptr))
            }
            #[inline]
            unsafe fn into_raw(self) -> *mut #itype {
                ComPtr::into_raw(Self::into_ptr(self))
            }
            #[inline]
            unsafe fn from_ptr(ptr: #ctype) -> Self {
                #ptr_wrap
            }
            #[inline]
            unsafe fn into_ptr(self) -> #ctype {
                self.#member
            }
        }
    }
}

fn dbg_impl(wrap: &Ident) -> TokenStream {
    let wrap_str = wrap.to_string();
    quote! {
        impl ::std::fmt::Debug for #wrap {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                fmt.debug_tuple(#wrap_str)
                    .field(&unsafe { Self::get_raw(self) })
                    .finish()
            }
        }
    }
}

fn meta_impl(wrap: &Ident, meta: &AttrInfo) -> TokenStream {
    let send_impl = if meta.send {
        quote! {
            unsafe impl Send for #wrap {}
        }
    } else {
        quote!{}
    };
    let sync_impl = if meta.sync {
        quote! {
            unsafe impl Sync for #wrap {}
        }
    } else {
        quote!{}
    };
    let dbg_impl = if meta.debug {
        dbg_impl(wrap)
    } else {
        quote!{}
    };

    quote! {
        #send_impl
        #sync_impl
        #dbg_impl
    }
}

fn create_wrapping(wrap: &Ident, member: &Member) -> TokenStream {
    match member {
        Member::Named(member) => quote! {
            #wrap { #member: ptr }
        },
        Member::Unnamed(_) => quote! {
            #wrap(ptr)
        },
    }
}

fn get_comptr_member(fields: &Fields) -> Result<(Member, &Type, &Type), String> {
    let (member, field) = match fields {
        Fields::Named(fields) => {
            if fields.named.len() != 1 {
                return Err("A ComWrapper struct must have exactly 1 member, a ComPtr.".into());
            }

            let field = &fields.named[0];
            let mem = Member::Named(field.ident.clone().unwrap());
            (mem, field)
        }
        Fields::Unnamed(fields) => {
            if fields.unnamed.len() != 1 {
                return Err("A ComWrapper struct must have exactly 1 member, a ComPtr.".into());
            }

            let field = &fields.unnamed[0];
            let mem = Member::Unnamed(Index {
                index: 0,
                span: field.span(),
            });
            (mem, field)
        }
        Fields::Unit => return Err("A ComWrapper struct must have a ComPtr member.".into()),
    };

    let itype = extract_comptr_ty(&field.ty)?;

    Ok((member, itype, &field.ty))
}

fn extract_comptr_ty(ty: &Type) -> Result<&Type, String> {
    let segments = match ty {
        Type::Path(typath) => &typath.path.segments,
        _ => return Err("A ComWrapper struct must have a ComPtr member.".into()),
    };

    let final_seg = match segments.last() {
        Some(seg) => *seg.value(),
        None => return Err("A ComWrapper struct must have a ComPtr member.".into()),
    };

    if final_seg.ident != "ComPtr" {
        return Err("A ComWrapper struct must have a ComPtr member.".into());
    }

    let args = match &final_seg.arguments {
        PathArguments::AngleBracketed(args) => &args.args,
        _ => return Err("Invalid generic arguments to ComPtr.".into()),
    };

    if args.len() != 1 {
        return Err("Invalid generic arguments to ComPtr.".into());
    }

    let itype = match &args[0] {
        GenericArgument::Type(ty) => ty,
        _ => return Err("Invalid generic arguments to ComPtr.".into()),
    };

    Ok(itype)
}

#[derive(Default)]
struct AttrInfo {
    send: bool,
    sync: bool,
    debug: bool,
}

fn parse_attr(attrs: &[Attribute]) -> Result<AttrInfo, String> {
    let com_attr = match attrs.iter().filter(is_com_attr).nth(0) {
        Some(attr) => attr,
        None => return Ok(Default::default()),
    };

    let meta = com_attr.parse_meta().map_err(|e| e.to_string())?;

    let attrs = match &meta {
        Meta::List(list) => &list.nested,
        _ => return Err("Invalid parameters to the `com` attribute".into()),
    };

    let mut send = false;
    let mut sync = false;
    let mut debug = false;
    for attr in attrs.iter() {
        let ident = match attr {
            NestedMeta::Meta(Meta::Word(ident)) => ident,
            _ => return Err("Invalid parameters to the `com` attribute".into()),
        };

        if ident == "send" {
            if send {
                return Err("Duplicate parameters to the `com` attribute".into());
            }
            send = true;
        } else if ident == "sync" {
            if sync {
                return Err("Duplicate parameters to the `com` attribute".into());
            }
            sync = true;
        } else if ident == "debug" {
            if debug {
                return Err("Duplicate parameters to the `com` attribute".into());
            }
            debug = true;
        } else {
            return Err("Invalid parameters to the `com` attribute".into());
        }
    }

    Ok(AttrInfo { send, sync, debug })
}

fn is_com_attr(attr: &&Attribute) -> bool {
    attr.path.segments.len() == 1 && attr.path.segments[0].ident == "com"
}
