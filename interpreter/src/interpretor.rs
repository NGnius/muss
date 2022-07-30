use super::lang::LanguageDictionary;

/// Builder function to add the standard statements parsers for MPS interpreters.
pub(crate) fn standard_vocab(vocabulary: &mut LanguageDictionary) {
    vocabulary
        // filters
        .add(crate::lang::vocabulary::filters::empty_filter())
        .add(crate::lang::vocabulary::filters::unique_filter()) // accepts .(unique)
        .add(crate::lang::vocabulary::filters::field_filter()) // accepts any .(something)
        .add(crate::lang::vocabulary::filters::field_filter_maybe())
        .add(crate::lang::vocabulary::filters::index_filter())
        .add(crate::lang::vocabulary::filters::range_filter())
        .add(crate::lang::vocabulary::filters::field_like_filter())
        .add(crate::lang::vocabulary::filters::field_re_filter())
        .add(crate::lang::vocabulary::filters::unique_field_filter())
        .add(crate::lang::vocabulary::filters::nonempty_filter())
        // sorters
        .add(crate::lang::vocabulary::sorters::empty_sort())
        .add(crate::lang::vocabulary::sorters::shuffle_sort()) // accepts ~(shuffle)
        .add(crate::lang::vocabulary::sorters::bliss_sort())
        .add(crate::lang::vocabulary::sorters::bliss_next_sort())
        .add(crate::lang::vocabulary::sorters::field_sort()) // accepts any ~(something)
        // iter blocks
        .add(
            crate::lang::ItemBlockFactory::new()
                .push(crate::lang::vocabulary::item_ops::ConstantItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::VariableAssignItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::FieldAssignItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::FileItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::VariableDeclareItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::InterpolateStringItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::BranchItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::IterItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::ConstructorItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::EmptyItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::RemoveItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::VariableRetrieveItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::NegateItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::NotItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::CompareItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::AddItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::SubtractItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::OrItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::AndItemOpFactory)
                .push(crate::lang::vocabulary::item_ops::BracketsItemOpFactory),
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
        .add(crate::lang::vocabulary::empty_function_factory())
        .add(crate::lang::vocabulary::empties_function_factory())
        .add(crate::lang::vocabulary::reset_function_factory())
        .add(crate::lang::vocabulary::union_function_factory())
        .add(crate::lang::vocabulary::intersection_function_factory());
}
