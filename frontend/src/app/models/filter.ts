export type FilterOperator =
  | 'eq' | 'ne'
  | 'in' | 'not_in'
  | 'contains' | 'not_contains'
  | 'gt' | 'gte' | 'lt' | 'lte';

export interface FilterExpression {
  field: string;
  operator: FilterOperator;
  value: string | string[];
}

export class FilterBuilder {
  private filters: FilterExpression[] = [];

  eq(field: string, value: string): this {
    this.filters.push({ field, operator: 'eq', value });
    return this;
  }

  ne(field: string, value: string): this {
    this.filters.push({ field, operator: 'ne', value });
    return this;
  }

  in(field: string, values: string[]): this {
    this.filters.push({ field, operator: 'in', value: values });
    return this;
  }

  notIn(field: string, values: string[]): this {
    this.filters.push({ field, operator: 'not_in', value: values });
    return this;
  }

  contains(field: string, value: string): this {
    this.filters.push({ field, operator: 'contains', value });
    return this;
  }

  notContains(field: string, value: string): this {
    this.filters.push({ field, operator: 'not_contains', value });
    return this;
  }

  gt(field: string, value: string): this {
    this.filters.push({ field, operator: 'gt', value });
    return this;
  }

  gte(field: string, value: string): this {
    this.filters.push({ field, operator: 'gte', value });
    return this;
  }

  lt(field: string, value: string): this {
    this.filters.push({ field, operator: 'lt', value });
    return this;
  }

  lte(field: string, value: string): this {
    this.filters.push({ field, operator: 'lte', value });
    return this;
  }

  build(): FilterExpression[] {
    return this.filters;
  }
}
