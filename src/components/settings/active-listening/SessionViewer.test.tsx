import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SessionViewer } from './SessionViewer';
import { commands } from '@/bindings';

// Get the mocked commands module
const mockCommands = vi.mocked(commands);

// Helper to create valid HistoryEntry mock data
const createMockEntry = (overrides: Partial<{
  id: number;
  file_name: string;
  timestamp: number;
  saved: boolean;
  title: string;
  transcription_text: string;
  post_processed_text: string | null;
  post_process_prompt: string | null;
}> = {}) => ({
  id: 1,
  file_name: 'recording.wav',
  timestamp: Date.now(),
  saved: false,
  title: 'Recording',
  transcription_text: 'Test transcription',
  post_processed_text: null,
  post_process_prompt: null,
  ...overrides,
});

describe('SessionViewer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('loading state', () => {
    it('shows loading message while fetching entries', async () => {
      // Keep the promise pending to test loading state
      mockCommands.getHistoryEntries.mockImplementation(
        () => new Promise(() => {})
      );

      render(<SessionViewer />);

      expect(screen.getByText('sessionViewer.loading')).toBeInTheDocument();
    });
  });

  describe('error state', () => {
    it('shows error message when fetch fails', async () => {
      mockCommands.getHistoryEntries.mockResolvedValueOnce({
        status: 'error',
        error: 'Failed to load',
      });

      render(<SessionViewer />);

      await waitFor(() => {
        expect(screen.getByText(/sessionViewer.error/)).toBeInTheDocument();
      });
    });
  });

  describe('empty state', () => {
    it('shows empty message when no sessions exist', async () => {
      mockCommands.getHistoryEntries.mockResolvedValueOnce({
        status: 'ok',
        data: [],
      });

      render(<SessionViewer />);

      await waitFor(() => {
        expect(screen.getByText('sessionViewer.empty')).toBeInTheDocument();
      });
    });

    it('shows empty state when entries have no post-processed text', async () => {
      mockCommands.getHistoryEntries.mockResolvedValueOnce({
        status: 'ok',
        data: [
          createMockEntry({
            id: 1,
            transcription_text: 'Test transcription',
            post_processed_text: null,
          }),
        ],
      });

      render(<SessionViewer />);

      await waitFor(() => {
        expect(screen.getByText('sessionViewer.empty')).toBeInTheDocument();
      });
    });
  });

  describe('with entries', () => {
    const mockEntries = [
      createMockEntry({
        id: 1,
        timestamp: Date.now(),
        transcription_text: 'First transcription',
        post_processed_text: 'First insight from AI',
      }),
      createMockEntry({
        id: 2,
        timestamp: Date.now() - 3600000, // 1 hour ago
        transcription_text: 'Second transcription',
        post_processed_text: 'Second insight from AI',
      }),
    ];

    it('displays entries with post-processed text', async () => {
      mockCommands.getHistoryEntries.mockResolvedValueOnce({
        status: 'ok',
        data: mockEntries,
      });

      render(<SessionViewer />);

      await waitFor(() => {
        expect(screen.getByText('sessionViewer.title')).toBeInTheDocument();
      });

      // Should show the transcription texts
      expect(screen.getByText(/First transcription/)).toBeInTheDocument();
      expect(screen.getByText(/Second transcription/)).toBeInTheDocument();
    });

    it('expands entry to show full details on click', async () => {
      const user = userEvent.setup();
      mockCommands.getHistoryEntries.mockResolvedValueOnce({
        status: 'ok',
        data: mockEntries,
      });

      render(<SessionViewer />);

      await waitFor(() => {
        expect(screen.getByText(/First transcription/)).toBeInTheDocument();
      });

      // Click on an entry to expand it
      const entryButton = screen.getByText(/First transcription/).closest('button');
      if (entryButton) {
        await user.click(entryButton);
      }

      // Should now show the AI insight
      await waitFor(() => {
        expect(screen.getByText('First insight from AI')).toBeInTheDocument();
      });
    });

    it('deletes entry when delete button is clicked', async () => {
      const user = userEvent.setup();
      mockCommands.getHistoryEntries.mockResolvedValueOnce({
        status: 'ok',
        data: mockEntries,
      });
      mockCommands.deleteHistoryEntry.mockResolvedValueOnce({
        status: 'ok',
        data: null,
      });

      render(<SessionViewer />);

      await waitFor(() => {
        expect(screen.getByText(/First transcription/)).toBeInTheDocument();
      });

      // Expand the entry first
      const entryButton = screen.getByText(/First transcription/).closest('button');
      if (entryButton) {
        await user.click(entryButton);
      }

      // Find and click the delete button
      await waitFor(() => {
        expect(screen.getByText('sessionViewer.delete')).toBeInTheDocument();
      });

      const deleteButton = screen.getByText('sessionViewer.delete');
      await user.click(deleteButton);

      expect(mockCommands.deleteHistoryEntry).toHaveBeenCalledWith(1);
    });
  });

  describe('grouping', () => {
    it('groups entries by date', async () => {
      const today = Date.now();
      const yesterday = today - 86400000; // 24 hours ago

      mockCommands.getHistoryEntries.mockResolvedValueOnce({
        status: 'ok',
        data: [
          createMockEntry({
            id: 1,
            timestamp: today,
            transcription_text: 'Today entry',
            post_processed_text: 'Today insight',
          }),
          createMockEntry({
            id: 2,
            timestamp: yesterday,
            transcription_text: 'Yesterday entry',
            post_processed_text: 'Yesterday insight',
          }),
        ],
      });

      render(<SessionViewer />);

      await waitFor(() => {
        expect(screen.getByText(/Today entry/)).toBeInTheDocument();
        expect(screen.getByText(/Yesterday entry/)).toBeInTheDocument();
      });
    });
  });
});
