# Task 6: Implement Search Command with Advanced Filtering

**GitHub Issue**: [#6](https://github.com/evcraddock/automark/issues/6)

## Objective
Create a comprehensive search command that leverages the repository's search and filtering capabilities.

## Requirements

1. **Create search command structure** in `src/commands/search.rs`:
   - SearchArgs struct with text query and filter options
   - Support for tag filtering (multiple tags with AND/OR logic)
   - Reading status filtering options
   - Priority range filtering
   - Date range filtering (bookmarked date, publish date)

2. **Implement SearchCommand**:
   - Use repository search_by_text and filtering methods
   - Combine multiple filter criteria effectively
   - Support case-insensitive text search
   - Return results with relevance or date sorting

3. **Define search argument patterns**:
   - Positional text query argument (optional)
   - --tags flag for tag filtering
   - --status flag for reading status filtering
   - --priority flag for priority range (e.g., "3-5")
   - --since and --until flags for date filtering
   - --sort flag for result ordering

4. **Implement result formatting**:
   - Human output: formatted table with highlighting
   - JSON output: structured results with metadata
   - Include search statistics (total matches, time taken)
   - Show partial matches and ranking when relevant

5. **Add search optimization**:
   - Implement efficient text search algorithms
   - Support stemming or fuzzy matching if beneficial
   - Cache frequently used search results
   - Handle large result sets with pagination

6. **Error handling for search**:
   - Handle invalid filter combinations
   - Provide helpful error messages for malformed queries
   - Handle empty results gracefully
   - Validate date ranges and priority ranges

7. **Write comprehensive tests** using TDD approach:
   - Test text search across all fields
   - Test individual and combined filters
   - Test result sorting and formatting
   - Test edge cases (empty results, invalid filters)
   - Test both JSON and human output formats

## Success Criteria
- Search command supports flexible query options
- Results are relevant and well-formatted
- Filtering works correctly in combination
- Performance is acceptable for large datasets
- Both output formats work properly