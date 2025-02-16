use proc_macro2::TokenStream;

// TODO Find a better place to put this
pub struct AliasMapArgs {
    pub map_path: syn::Path,
    pub map_path_comma: syn::Token![,],
    pub vis: syn::Visibility,
    pub unique_ident: syn::Ident,
    pub generics: syn::Generics,
    pub where_clause: Option<syn::WhereClause>,
    pub unique_ident_comma: syn::Token![,],
    pub telety_path: Option<syn::Path>,
    pub telety_path_comma: syn::Token![,],
    pub self_type: Option<syn::Type>,
    pub self_type_comma: syn::Token![,],
    pub aliased_types: syn::punctuated::Punctuated<syn::Type, syn::Token![,]>,
}

impl AliasMapArgs {
    pub fn new(
        map_path: syn::Path, 
        vis: syn::Visibility, 
        unique_ident: syn::Ident,
        mut generics: syn::Generics,
        telety_path: Option<syn::Path>,
        self_type: Option<syn::Type>,
        aliased_types: Vec<syn::Type>,
    ) -> Self {
        let where_clause = generics.where_clause.take();

        Self {
            map_path,
            map_path_comma: Default::default(),
            vis,
            unique_ident,
            generics,
            where_clause,
            unique_ident_comma: Default::default(),
            telety_path,
            telety_path_comma: Default::default(),
            self_type,
            self_type_comma: Default::default(),
            aliased_types: aliased_types.into_iter().collect(),
        }
    }
}

impl quote::ToTokens for AliasMapArgs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            map_path,
            map_path_comma,
            vis,
            unique_ident,
            generics,
            where_clause,
            unique_ident_comma,
            telety_path,
            telety_path_comma,
            self_type,
            self_type_comma,
            aliased_types,
        } = self;

        map_path.to_tokens(tokens);
        map_path_comma.to_tokens(tokens);
        vis.to_tokens(tokens);
        unique_ident.to_tokens(tokens);
        generics.to_tokens(tokens);
        where_clause.to_tokens(tokens);
        unique_ident_comma.to_tokens(tokens);
        telety_path.to_tokens(tokens);
        telety_path_comma.to_tokens(tokens);
        self_type.to_tokens(tokens);
        self_type_comma.to_tokens(tokens);
        aliased_types.to_tokens(tokens);
    }
}

impl syn::parse::Parse for AliasMapArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let map_path = input.parse()?;
        let map_path_comma = input.parse()?;
        let vis = input.parse()?;
        let unique_ident = input.parse()?;
        let generics = input.parse()?;
        let where_clause = 
            input.peek(syn::Token![where])
                .then(|| input.parse())
                .transpose()?;
        let unique_ident_comma = input.parse()?;
        let telety_path = 
            (!input.peek(syn::Token![,]))
                .then(|| input.parse())
                .transpose()?;
        let telety_path_comma = input.parse()?;
        let self_type = 
            (!input.peek(syn::Token![,]))
                .then(|| input.parse())
                .transpose()?;
        let self_type_comma = input.parse()?;
        let aliased_types = syn::punctuated::Punctuated::parse_terminated(input)?;

        Ok(Self {
            map_path,
            map_path_comma,
            vis,
            unique_ident,
            generics,
            where_clause,
            unique_ident_comma,
            telety_path,
            telety_path_comma,
            self_type,
            self_type_comma,
            aliased_types,
        })
    }
}