use super::lang::LanguageDictionary;

/// Builder function to add the standard statements parsers for MPS interpreters.
pub(crate) fn standard_vocab(vocabulary: &mut LanguageDictionary) {
    vocabulary
        // filters
        .add_transform(crate::lang::vocabulary::filters::empty_filter())
        .add_transform(crate::lang::vocabulary::filters::range_filter())
        .add_transform( // accepts any .(.something)
            crate::lang::vocabulary::filters::field::FieldFilterBlockFactory::new()
                .push(crate::lang::vocabulary::filters::field::FieldFilterComparisonFactory)
                .push(crate::lang::vocabulary::filters::field::FieldFilterMaybeFactory)
                .push(crate::lang::vocabulary::filters::field::FieldLikeFilterFactory)
                .push(crate::lang::vocabulary::filters::field::FieldRegexFilterFactory)
                .to_statement_factory()
        )
        .add_transform(crate::lang::vocabulary::filters::unique_field_filter())
        .add_transform(crate::lang::vocabulary::filters::unique_filter())
        .add_transform(crate::lang::vocabulary::filters::nonempty_filter())
        .add_transform(crate::lang::vocabulary::filters::index_filter())
        // sorters
        .add_transform(crate::lang::vocabulary::sorters::empty_sort())
        .add_transform(crate::lang::vocabulary::sorters::shuffle_sort()) // accepts ~(~shuffle)
        .add_transform(crate::lang::vocabulary::sorters::bliss_sort())
        .add_transform(crate::lang::vocabulary::sorters::bliss_next_sort())
        .add_transform(crate::lang::vocabulary::sorters::radio_sort())
        .add_transform(crate::lang::vocabulary::sorters::field_sort()) // accepts any ~(.name)
        // iter blocks
        .add_transform(
            crate::lang::ItemBlockFactory::new()
                .push(crate::lang::vocabulary::item_ops::FieldAssignItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::VariableAssignItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::VariableDeclareItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::FileItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::InterpolateStringItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::BranchItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::IterItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::ConstructorItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::EmptyItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::RemoveItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::NotItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::CompareItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::NegateItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::AddItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::SubtractItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::OrItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::AndItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::BracketsItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::FieldRetrieveItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::ConstantItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::VariableRetrieveItemOpFactory)
        )
        // functions and misc
        // functions don't enforce bracket coherence
        // -- function().() is valid despite the ).( in between brackets
        .add(crate::lang::vocabulary::sql_function_factory())
        .add(crate::lang::vocabulary::mpd_query_function_factory())
        .add(crate::lang::vocabulary::simple_sql_function_factory())
        .add(crate::lang::vocabulary::repeat_function_factory())
        .add(crate::lang::vocabulary::AssignStatementFactory)
        .add(crate::lang::vocabulary::sql_init_function_factory())
        .add(crate::lang::vocabulary::files_function_factory())
        .add(crate::lang::vocabulary::playlist_function_factory())
        .add(crate::lang::vocabulary::empty_function_factory())
        .add(crate::lang::vocabulary::empties_function_factory())
        .add(crate::lang::vocabulary::reset_function_factory())
        .add(crate::lang::vocabulary::union_function_factory())
        .add(crate::lang::vocabulary::intersection_function_factory())
        .add(crate::lang::vocabulary::VariableRetrieveStatementFactory);
}
