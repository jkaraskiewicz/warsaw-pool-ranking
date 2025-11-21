# Warsaw Pool Rankings - Tests

Unit tests for the rating algorithm and core functionality.

## Running Tests

### Run all tests
```bash
pytest
```

### Run specific test file
```bash
pytest tests/test_calculator.py
pytest tests/test_time_decay.py
pytest tests/test_confidence.py
```

### Run specific test class
```bash
pytest tests/test_calculator.py::TestRatingCalculator
```

### Run specific test
```bash
pytest tests/test_calculator.py::TestRatingCalculator::test_single_game
```

### Run with verbose output
```bash
pytest -v
```

### Run with coverage
```bash
pytest --cov=app --cov-report=html
```

Then open `htmlcov/index.html` in a browser to see coverage report.

## Test Structure

- `test_calculator.py` - Tests for Bradley-Terry ML rating calculator
  - Basic functionality
  - Rating scale calibration (100 pts = 2:1 odds)
  - Win probability predictions
  - Time-weighted calculations

- `test_time_decay.py` - Tests for exponential time decay
  - Weight calculations
  - Half-life verification
  - Edge cases (future dates, zero days)

- `test_confidence.py` - Tests for player confidence and blending
  - Confidence level determination
  - Rating blending for new players
  - Threshold behavior

## Test Coverage

Current test coverage focuses on:
- ✓ Rating calculator (Bradley-Terry ML)
- ✓ Time decay (exponential)
- ✓ Player confidence and blending
- TODO: Weekly simulator
- TODO: API endpoints (integration tests)
- TODO: Data parsers

## Writing New Tests

Follow pytest conventions:
- Test files: `test_*.py`
- Test classes: `Test*`
- Test functions: `test_*`

Example:
```python
import pytest
from app.rating.calculator import RatingCalculator

class TestRatingCalculator:
    def test_something(self):
        calc = RatingCalculator()
        # ... test code ...
        assert result == expected
```

## Continuous Integration

These tests should be run:
- Before committing code
- In CI/CD pipeline
- Before deploying to production

Ensure all tests pass before merging pull requests.
