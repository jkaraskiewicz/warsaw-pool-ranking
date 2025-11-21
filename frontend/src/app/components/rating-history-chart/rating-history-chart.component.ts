import { Component, Input, OnChanges, SimpleChanges } from '@angular/core';
import { ChartConfiguration, ChartOptions } from 'chart.js';
import { RatingSnapshot } from '../../models/player.model';

@Component({
  selector: 'app-rating-history-chart',
  templateUrl: './rating-history-chart.component.html',
  styleUrls: ['./rating-history-chart.component.scss']
})
export class RatingHistoryChartComponent implements OnChanges {
  @Input() history: RatingSnapshot[] = [];

  public lineChartData: ChartConfiguration<'line'>['data'] = {
    labels: [],
    datasets: [
      {
        data: [],
        label: 'Rating',
        fill: true,
        tension: 0.3,
        borderColor: '#3f51b5',
        backgroundColor: 'rgba(63, 81, 181, 0.1)',
        pointBackgroundColor: '#3f51b5',
        pointBorderColor: '#fff',
        pointHoverBackgroundColor: '#fff',
        pointHoverBorderColor: '#3f51b5',
      }
    ]
  };

  public lineChartOptions: ChartOptions<'line'> = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        display: false
      },
      tooltip: {
        callbacks: {
          label: (context) => {
            return `Rating: ${context.parsed.y?.toFixed(0) || '0'}`;
          }
        }
      }
    },
    scales: {
      x: {
        display: true,
        title: {
          display: true,
          text: 'Week Ending'
        }
      },
      y: {
        display: true,
        title: {
          display: true,
          text: 'Rating'
        },
        ticks: {
          callback: (value) => {
            return Math.round(value as number).toString();
          }
        }
      }
    }
  };

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['history'] && this.history) {
      this.updateChart();
    }
  }

  private updateChart(): void {
    // Sort history by date
    const sortedHistory = [...this.history].sort((a, b) =>
      new Date(a.week_ending).getTime() - new Date(b.week_ending).getTime()
    );

    // Format dates for labels
    const labels = sortedHistory.map(snapshot =>
      new Date(snapshot.week_ending).toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: '2-digit'
      })
    );

    // Extract ratings
    const ratings = sortedHistory.map(snapshot => snapshot.rating);

    // Update chart data
    this.lineChartData = {
      labels: labels,
      datasets: [
        {
          ...this.lineChartData.datasets[0],
          data: ratings
        }
      ]
    };
  }
}
