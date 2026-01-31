import { TestBed } from '@angular/core/testing';
import { of } from 'rxjs';
import { PlayerListFacade, PlayerListState } from './player-list.facade';
import { PlayerService } from '../../services/player.service';
import { InterestedPlayersService } from '../../services/interested-players.service';
import { PlayerListResponse, PlayerListItem } from '../../models/api';
import { signal } from '@angular/core';

describe('PlayerListFacade', () => {
  let facade: PlayerListFacade;
  let playerServiceSpy: jasmine.SpyObj<PlayerService>;
  let interestedPlayersServiceSpy: jasmine.SpyObj<InterestedPlayersService>;

  const mockPlayer: PlayerListItem = {
    rank: 1,
    playerId: 123,
    cuescoreId: 456,
    name: 'Test Player',
    rating: 550,
    gamesPlayed: 100,
    confidenceLevel: 'established',
    matchesPlayed: 50
  };

  const mockResponse: PlayerListResponse = {
    items: [mockPlayer],
    total: 1,
    page: 1,
    pageSize: 100
  };

  beforeEach(() => {
    const playerSpy = jasmine.createSpyObj('PlayerService', ['getPlayers']);
    const interestedSpy = jasmine.createSpyObj('InterestedPlayersService', ['interestedPlayerIds'], {
      interestedPlayerIds: signal(new Set<number>())
    });

    playerSpy.getPlayers.and.returnValue(of(mockResponse));

    TestBed.configureTestingModule({
      providers: [
        PlayerListFacade,
        { provide: PlayerService, useValue: playerSpy },
        { provide: InterestedPlayersService, useValue: interestedSpy }
      ]
    });

    playerServiceSpy = TestBed.inject(PlayerService) as jasmine.SpyObj<PlayerService>;
    interestedPlayersServiceSpy = TestBed.inject(InterestedPlayersService) as jasmine.SpyObj<InterestedPlayersService>;
    facade = TestBed.inject(PlayerListFacade);
  });

  describe('initial state', () => {
    it('should have correct default state values', () => {
      const state = facade.state();

      expect(state.pageIndex).toBe(0);
      expect(state.pageSize).toBe(100);
      expect(state.sortActive).toBe('rating');
      expect(state.sortDirection).toBe('desc');
      expect(state.nameFilter).toBe('');
      expect(state.ratingType).toBe('active');
      expect(state.showInterestedOnly).toBe(false);
    });

    it('should start with loading true', () => {
      expect(facade.loading()).toBe(true);
    });

    it('should start with empty players array', () => {
      expect(facade.players()).toEqual([]);
    });

    it('should start with total 0', () => {
      expect(facade.total()).toBe(0);
    });
  });

  describe('state update methods', () => {
    describe('setPage', () => {
      it('should update page index and page size', () => {
        facade.setPage(2, 50);
        const state = facade.state();

        expect(state.pageIndex).toBe(2);
        expect(state.pageSize).toBe(50);
      });

      it('should handle page 0', () => {
        facade.setPage(0, 100);
        expect(facade.state().pageIndex).toBe(0);
      });

      it('should handle large page numbers', () => {
        facade.setPage(999, 100);
        expect(facade.state().pageIndex).toBe(999);
      });
    });

    describe('setSort', () => {
      it('should update sort active and direction', () => {
        facade.setSort('name', 'asc');
        const state = facade.state();

        expect(state.sortActive).toBe('name');
        expect(state.sortDirection).toBe('asc');
      });

      it('should update sort to descending', () => {
        facade.setSort('games_played', 'desc');
        const state = facade.state();

        expect(state.sortActive).toBe('games_played');
        expect(state.sortDirection).toBe('desc');
      });
    });

    describe('setFilter', () => {
      it('should update name filter', () => {
        facade.setFilter('test');
        expect(facade.state().nameFilter).toBe('test');
      });

      it('should reset page index to 0', () => {
        facade.setPage(5, 100);
        expect(facade.state().pageIndex).toBe(5);

        facade.setFilter('test');
        expect(facade.state().pageIndex).toBe(0);
      });

      it('should handle empty filter', () => {
        facade.setFilter('something');
        facade.setFilter('');
        expect(facade.state().nameFilter).toBe('');
      });

      it('should handle special characters in filter', () => {
        facade.setFilter("O'Brien");
        expect(facade.state().nameFilter).toBe("O'Brien");
      });
    });

    describe('setRatingType', () => {
      it('should update rating type', () => {
        facade.setRatingType('1y');
        expect(facade.state().ratingType).toBe('1y');
      });

      it('should reset page index to 0', () => {
        facade.setPage(3, 100);
        expect(facade.state().pageIndex).toBe(3);

        facade.setRatingType('1y');
        expect(facade.state().pageIndex).toBe(0);
      });

      it('should handle all rating types', () => {
        const ratingTypes = ['all', '1y', '2y', '3y', '4y', '5y', 'active'];
        for (const type of ratingTypes) {
          facade.setRatingType(type);
          expect(facade.state().ratingType).toBe(type);
        }
      });
    });

    describe('setShowInterestedOnly', () => {
      it('should update show interested flag to true', () => {
        facade.setShowInterestedOnly(true);
        expect(facade.state().showInterestedOnly).toBe(true);
      });

      it('should update show interested flag to false', () => {
        facade.setShowInterestedOnly(true);
        facade.setShowInterestedOnly(false);
        expect(facade.state().showInterestedOnly).toBe(false);
      });

      it('should reset page index to 0', () => {
        facade.setPage(2, 100);
        expect(facade.state().pageIndex).toBe(2);

        facade.setShowInterestedOnly(true);
        expect(facade.state().pageIndex).toBe(0);
      });
    });
  });

  describe('state immutability', () => {
    it('should not mutate previous state on update', () => {
      const initialState = facade.state();

      facade.setFilter('new filter');

      expect(initialState.nameFilter).toBe('');
      expect(facade.state().nameFilter).toBe('new filter');
    });

    it('should preserve unchanged properties', () => {
      facade.setSort('rating', 'desc');
      facade.setPage(5, 50);

      facade.setFilter('test');

      const state = facade.state();
      expect(state.sortActive).toBe('rating');
      expect(state.sortDirection).toBe('desc');
      expect(state.pageSize).toBe(50);
      // Page index resets on filter change
      expect(state.pageIndex).toBe(0);
    });
  });

  describe('multiple rapid updates', () => {
    it('should handle multiple consecutive updates', () => {
      facade.setFilter('a');
      facade.setFilter('ab');
      facade.setFilter('abc');

      expect(facade.state().nameFilter).toBe('abc');
    });

    it('should handle mixed updates', () => {
      facade.setFilter('test');
      facade.setRatingType('1y');
      facade.setShowInterestedOnly(true);
      facade.setSort('name', 'asc');

      const state = facade.state();
      expect(state.nameFilter).toBe('test');
      expect(state.ratingType).toBe('1y');
      expect(state.showInterestedOnly).toBe(true);
      expect(state.sortActive).toBe('name');
      expect(state.sortDirection).toBe('asc');
    });
  });
});
