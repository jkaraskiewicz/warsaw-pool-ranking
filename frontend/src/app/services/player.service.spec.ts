import { TestBed } from '@angular/core/testing';
import { HttpClientTestingModule, HttpTestingController } from '@angular/common/http/testing';
import { PlayerService } from './player.service';
import { FilterBuilder } from '../models/filter';
import { PlayerListResponse, PlayerDetail, HeadToHeadResponse, PlayerRivalriesResponse } from '../models/api';

describe('PlayerService', () => {
  let service: PlayerService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [HttpClientTestingModule],
      providers: [PlayerService]
    });
    service = TestBed.inject(PlayerService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    httpMock.verify();
  });

  describe('getPlayers', () => {
    it('should fetch players with default parameters', () => {
      const mockResponse: PlayerListResponse = {
        items: [],
        total: 0,
        page: 1,
        pageSize: 100
      };

      service.getPlayers().subscribe(response => {
        expect(response).toEqual(mockResponse);
      });

      const req = httpMock.expectOne(r =>
        r.url === '/api/players' &&
        r.params.get('page') === '1' &&
        r.params.get('page_size') === '100' &&
        r.params.get('sort_by') === 'rating' &&
        r.params.get('order') === 'desc'
      );
      expect(req.request.method).toBe('GET');
      req.flush(mockResponse);
    });

    it('should include pagination parameters', () => {
      service.getPlayers(2, 50).subscribe(response => {
        expect(response.page).toBe(2);
        expect(response.pageSize).toBe(50);
      });

      const req = httpMock.expectOne(r =>
        r.params.get('page') === '2' &&
        r.params.get('page_size') === '50'
      );
      req.flush({ items: [], total: 0, page: 2, pageSize: 50 });
    });

    it('should include sort parameters', () => {
      service.getPlayers(1, 100, 'name', 'asc').subscribe(response => {
        expect(response).toBeDefined();
      });

      const req = httpMock.expectOne(r =>
        r.params.get('sort_by') === 'name' &&
        r.params.get('order') === 'asc'
      );
      expect(req.request.params.get('sort_by')).toBe('name');
      expect(req.request.params.get('order')).toBe('asc');
      req.flush({ items: [], total: 0, page: 1, pageSize: 100 });
    });

    it('should build filter DSL from single filter', () => {
      const filters = new FilterBuilder().eq('rating_type', 'all').build();
      service.getPlayers(1, 100, 'rating', 'desc', filters).subscribe(response => {
        expect(response).toBeDefined();
      });

      const req = httpMock.expectOne(r =>
        r.params.get('filter') === 'rating_type:eq:all'
      );
      expect(req.request.params.get('filter')).toBe('rating_type:eq:all');
      req.flush({ items: [], total: 0, page: 1, pageSize: 100 });
    });

    it('should build filter DSL from multiple filters', () => {
      const filters = new FilterBuilder()
        .eq('rating_type', 'all')
        .contains('name', 'Jan')
        .build();
      service.getPlayers(1, 100, 'rating', 'desc', filters).subscribe(response => {
        expect(response).toBeDefined();
      });

      const req = httpMock.expectOne(r =>
        r.params.get('filter') === 'rating_type:eq:all|name:contains:Jan'
      );
      expect(req.request.params.get('filter')).toBe('rating_type:eq:all|name:contains:Jan');
      req.flush({ items: [], total: 0, page: 1, pageSize: 100 });
    });

    it('should build filter DSL with array values (in operator)', () => {
      const filters = new FilterBuilder().in('id', ['1', '5', '10']).build();
      service.getPlayers(1, 100, 'rating', 'desc', filters).subscribe(response => {
        expect(response).toBeDefined();
      });

      const req = httpMock.expectOne(r =>
        r.params.get('filter') === 'id:in:1,5,10'
      );
      expect(req.request.params.get('filter')).toBe('id:in:1,5,10');
      req.flush({ items: [], total: 0, page: 1, pageSize: 100 });
    });

    it('should not include filter param when filters are empty', () => {
      service.getPlayers(1, 100, 'rating', 'desc', []).subscribe(response => {
        expect(response).toBeDefined();
      });

      const req = httpMock.expectOne(r => !r.params.has('filter'));
      expect(req.request.params.has('filter')).toBe(false);
      req.flush({ items: [], total: 0, page: 1, pageSize: 100 });
    });
  });

  describe('getPlayerDetail', () => {
    it('should fetch player details with default rating type', () => {
      const mockPlayer: PlayerDetail = {
        playerId: 123,
        cuescoreId: 456,
        name: 'Test Player',
        cuescoreProfileUrl: 'https://cuescore.com/player/456',
        rating: 550,
        gamesPlayed: 100,
        confidenceLevel: 'established',
        mlRating: 560,
        starterWeight: 0.1,
        mlWeight: 0.9,
        effectiveGames: 90,
        matchesPlayed: 50,
        lastMatches: []
      };

      service.getPlayerDetail(123).subscribe(player => {
        expect(player).toEqual(mockPlayer);
      });

      const req = httpMock.expectOne(r =>
        r.url === '/api/player/123' &&
        r.params.get('rating_type') === 'all'
      );
      expect(req.request.method).toBe('GET');
      req.flush(mockPlayer);
    });

    it('should fetch player details with custom rating type', () => {
      service.getPlayerDetail(123, '1y').subscribe(response => {
        expect(response).toBeDefined();
      });

      const req = httpMock.expectOne(r =>
        r.url === '/api/player/123' &&
        r.params.get('rating_type') === '1y'
      );
      expect(req.request.method).toBe('GET');
      req.flush({});
    });
  });

  describe('getHeadToHeadComparison', () => {
    it('should fetch head to head comparison', () => {
      const mockResponse: HeadToHeadResponse = {
        player1: undefined,
        player2: undefined,
        probabilityP1Wins: 0.6,
        matches: [],
        stats: undefined
      };

      service.getHeadToHeadComparison(1, 2).subscribe(response => {
        expect(response.probabilityP1Wins).toBe(0.6);
      });

      const req = httpMock.expectOne(r =>
        r.url === '/api/compare/1/2' &&
        r.params.get('rating_type') === 'all'
      );
      expect(req.request.method).toBe('GET');
      req.flush(mockResponse);
    });

    it('should include custom rating type', () => {
      service.getHeadToHeadComparison(1, 2, '2y').subscribe(response => {
        expect(response).toBeDefined();
      });

      const req = httpMock.expectOne(r =>
        r.params.get('rating_type') === '2y'
      );
      expect(req.request.method).toBe('GET');
      req.flush({});
    });
  });

  describe('getPlayerRivalries', () => {
    it('should fetch player rivalries', () => {
      const mockResponse: PlayerRivalriesResponse = {
        nemeses: [{ opponentId: 2, opponentName: 'Nemesis', matchesPlayed: 10, matchesWon: 2, winRate: 0.2 }],
        bunnies: [{ opponentId: 3, opponentName: 'Bunny', matchesPlayed: 10, matchesWon: 8, winRate: 0.8 }]
      };

      service.getPlayerRivalries(1).subscribe(response => {
        expect(response.nemeses.length).toBe(1);
        expect(response.bunnies.length).toBe(1);
      });

      const req = httpMock.expectOne('/api/player/1/rivalries');
      expect(req.request.method).toBe('GET');
      req.flush(mockResponse);
    });
  });
});
