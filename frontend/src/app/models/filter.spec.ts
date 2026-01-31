import { FilterBuilder, FilterExpression } from './filter';

describe('FilterBuilder', () => {
  let builder: FilterBuilder;

  beforeEach(() => {
    builder = new FilterBuilder();
  });

  describe('single filter methods', () => {
    it('should build eq filter', () => {
      const result = builder.eq('rating_type', 'all').build();

      expect(result.length).toBe(1);
      expect(result[0]).toEqual({
        field: 'rating_type',
        operator: 'eq',
        value: 'all'
      });
    });

    it('should build ne filter', () => {
      const result = builder.ne('status', 'inactive').build();

      expect(result.length).toBe(1);
      expect(result[0].operator).toBe('ne');
    });

    it('should build contains filter', () => {
      const result = builder.contains('name', 'ski').build();

      expect(result[0]).toEqual({
        field: 'name',
        operator: 'contains',
        value: 'ski'
      });
    });

    it('should build notContains filter', () => {
      const result = builder.notContains('name', 'test').build();

      expect(result[0].operator).toBe('not_contains');
    });

    it('should build in filter with array values', () => {
      const result = builder.in('id', ['1', '5', '10']).build();

      expect(result[0]).toEqual({
        field: 'id',
        operator: 'in',
        value: ['1', '5', '10']
      });
    });

    it('should build notIn filter with array values', () => {
      const result = builder.notIn('id', ['3', '7']).build();

      expect(result[0].operator).toBe('not_in');
      expect(result[0].value).toEqual(['3', '7']);
    });
  });

  describe('comparison operators', () => {
    it('should build gt filter', () => {
      const result = builder.gt('rating', '600').build();
      expect(result[0].operator).toBe('gt');
    });

    it('should build gte filter', () => {
      const result = builder.gte('games_played', '50').build();
      expect(result[0].operator).toBe('gte');
    });

    it('should build lt filter', () => {
      const result = builder.lt('rating', '400').build();
      expect(result[0].operator).toBe('lt');
    });

    it('should build lte filter', () => {
      const result = builder.lte('games_played', '10').build();
      expect(result[0].operator).toBe('lte');
    });
  });

  describe('chaining', () => {
    it('should support method chaining', () => {
      const result = builder
        .eq('rating_type', 'active')
        .contains('name', 'Jan')
        .gt('rating', '500')
        .build();

      expect(result.length).toBe(3);
      expect(result[0].field).toBe('rating_type');
      expect(result[1].field).toBe('name');
      expect(result[2].field).toBe('rating');
    });

    it('should return this from each method', () => {
      const returnedBuilder = builder.eq('field', 'value');
      expect(returnedBuilder).toBe(builder);
    });
  });

  describe('build', () => {
    it('should return empty array when no filters added', () => {
      const result = builder.build();
      expect(result).toEqual([]);
    });

    it('should preserve order of filters', () => {
      const result = builder
        .eq('a', '1')
        .eq('b', '2')
        .eq('c', '3')
        .build();

      expect(result[0].field).toBe('a');
      expect(result[1].field).toBe('b');
      expect(result[2].field).toBe('c');
    });
  });
});
