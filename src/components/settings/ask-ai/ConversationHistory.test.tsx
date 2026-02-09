import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ConversationHistory } from './ConversationHistory';
import { commands } from '@/bindings';

// Get the mocked commands module
const mockCommands = vi.mocked(commands);

// Helper to create valid ConversationTurn mock data
const createMockTurn = (overrides: Partial<{
  id: string;
  question: string;
  response: string;
  timestamp: number;
  audio_file_name: string | null;
}> = {}) => ({
  id: 'turn-1',
  question: 'Test question',
  response: 'Test response',
  timestamp: Date.now(),
  audio_file_name: null,
  ...overrides,
});

// Helper to create valid AskAiConversation mock data
const createMockConversation = (overrides: Partial<{
  id: string;
  title: string | null;
  created_at: number;
  updated_at: number;
  turns: ReturnType<typeof createMockTurn>[];
}> = {}) => ({
  id: 'conv-1',
  title: 'Test Conversation',
  created_at: Date.now(),
  updated_at: Date.now(),
  turns: [createMockTurn()],
  ...overrides,
});

describe('ConversationHistory', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('loading state', () => {
    it('shows loading message while fetching conversations', async () => {
      // Keep the promise pending to test loading state
      mockCommands.listAskAiConversations.mockImplementation(
        () => new Promise(() => {})
      );

      render(<ConversationHistory />);

      expect(screen.getByText('askAi.history.loading')).toBeInTheDocument();
    });
  });

  describe('error state', () => {
    it('shows error message when fetch fails', async () => {
      mockCommands.listAskAiConversations.mockResolvedValueOnce({
        status: 'error',
        error: 'Failed to load',
      });

      render(<ConversationHistory />);

      await waitFor(() => {
        expect(screen.getByText(/askAi.history.error/)).toBeInTheDocument();
      });
    });
  });

  describe('empty state', () => {
    it('shows empty message when no conversations exist', async () => {
      mockCommands.listAskAiConversations.mockResolvedValueOnce({
        status: 'ok',
        data: [],
      });

      render(<ConversationHistory />);

      await waitFor(() => {
        expect(screen.getByText('askAi.history.empty')).toBeInTheDocument();
      });
    });
  });

  describe('with conversations', () => {
    const mockConversations = [
      createMockConversation({
        id: 'conv-1',
        title: 'First Conversation',
        created_at: Date.now(),
        updated_at: Date.now(),
        turns: [
          createMockTurn({
            id: 'turn-1',
            question: 'What is the weather?',
            response: 'The weather is sunny today.',
            timestamp: Date.now(),
          }),
        ],
      }),
      createMockConversation({
        id: 'conv-2',
        title: 'Second Conversation',
        created_at: Date.now() - 3600000,
        updated_at: Date.now() - 3600000,
        turns: [
          createMockTurn({
            id: 'turn-2',
            question: 'How do I cook pasta?',
            response: 'Boil water, add salt, cook for 8-10 minutes.',
            timestamp: Date.now() - 3600000,
          }),
          createMockTurn({
            id: 'turn-3',
            question: 'What about sauce?',
            response: 'You can add tomato sauce or pesto.',
            timestamp: Date.now() - 3500000,
          }),
        ],
      }),
    ];

    it('displays conversation titles', async () => {
      mockCommands.listAskAiConversations.mockResolvedValueOnce({
        status: 'ok',
        data: mockConversations,
      });

      render(<ConversationHistory />);

      await waitFor(() => {
        expect(screen.getByText('First Conversation')).toBeInTheDocument();
        expect(screen.getByText('Second Conversation')).toBeInTheDocument();
      });
    });

    it('shows turn count for each conversation', async () => {
      mockCommands.listAskAiConversations.mockResolvedValueOnce({
        status: 'ok',
        data: mockConversations,
      });

      render(<ConversationHistory />);

      await waitFor(() => {
        expect(screen.getByText('First Conversation')).toBeInTheDocument();
      });

      // Check for turn counts using the translation key pattern
      // The actual text is "askAi.conversation.turns" with count interpolation
      const turnElements = screen.getAllByText(/turns/i);
      expect(turnElements.length).toBeGreaterThan(0);
    });

    it('expands conversation to show full details on click', async () => {
      const user = userEvent.setup();
      mockCommands.listAskAiConversations.mockResolvedValueOnce({
        status: 'ok',
        data: mockConversations,
      });

      render(<ConversationHistory />);

      await waitFor(() => {
        expect(screen.getByText('First Conversation')).toBeInTheDocument();
      });

      // Click on the first conversation to expand it
      const conversationButton = screen
        .getByText('First Conversation')
        .closest('button');
      if (conversationButton) {
        await user.click(conversationButton);
      }

      // Should now show the question and answer
      await waitFor(() => {
        expect(screen.getByText('What is the weather?')).toBeInTheDocument();
        expect(
          screen.getByText('The weather is sunny today.')
        ).toBeInTheDocument();
      });
    });

    it('deletes conversation when delete button is clicked', async () => {
      const user = userEvent.setup();
      mockCommands.listAskAiConversations.mockResolvedValueOnce({
        status: 'ok',
        data: mockConversations,
      });
      mockCommands.deleteAskAiConversationFromHistory.mockResolvedValueOnce({
        status: 'ok',
        data: null,
      });

      render(<ConversationHistory />);

      await waitFor(() => {
        expect(screen.getByText('First Conversation')).toBeInTheDocument();
      });

      // Find delete buttons (there should be one per conversation)
      const deleteButtons = screen.getAllByRole('button').filter((btn) => {
        const svg = btn.querySelector('svg');
        return svg && btn.className.includes('red');
      });

      expect(deleteButtons.length).toBeGreaterThan(0);
      await user.click(deleteButtons[0]);

      expect(
        mockCommands.deleteAskAiConversationFromHistory
      ).toHaveBeenCalledWith('conv-1');
    });
  });

  describe('search functionality', () => {
    const mockConversations = [
      createMockConversation({
        id: 'conv-1',
        title: 'Weather Questions',
        created_at: Date.now(),
        updated_at: Date.now(),
        turns: [
          createMockTurn({
            id: 'turn-1',
            question: 'What is the weather?',
            response: 'It is sunny.',
            timestamp: Date.now(),
          }),
        ],
      }),
      createMockConversation({
        id: 'conv-2',
        title: 'Cooking Tips',
        created_at: Date.now(),
        updated_at: Date.now(),
        turns: [
          createMockTurn({
            id: 'turn-2',
            question: 'How to cook rice?',
            response: 'Use 1:2 ratio of rice to water.',
            timestamp: Date.now(),
          }),
        ],
      }),
    ];

    it('filters conversations by search query on title', async () => {
      const user = userEvent.setup();
      mockCommands.listAskAiConversations.mockResolvedValueOnce({
        status: 'ok',
        data: mockConversations,
      });

      render(<ConversationHistory />);

      await waitFor(() => {
        expect(screen.getByText('Weather Questions')).toBeInTheDocument();
        expect(screen.getByText('Cooking Tips')).toBeInTheDocument();
      });

      // Type in search box
      const searchInput = screen.getByPlaceholderText('askAi.history.search');
      await user.type(searchInput, 'Weather');

      // Only weather conversation should be visible
      expect(screen.getByText('Weather Questions')).toBeInTheDocument();
      expect(screen.queryByText('Cooking Tips')).not.toBeInTheDocument();
    });

    it('filters conversations by search query on content', async () => {
      const user = userEvent.setup();
      mockCommands.listAskAiConversations.mockResolvedValueOnce({
        status: 'ok',
        data: mockConversations,
      });

      render(<ConversationHistory />);

      await waitFor(() => {
        expect(screen.getByText('Weather Questions')).toBeInTheDocument();
        expect(screen.getByText('Cooking Tips')).toBeInTheDocument();
      });

      // Search for content in the question
      const searchInput = screen.getByPlaceholderText('askAi.history.search');
      await user.type(searchInput, 'rice');

      // Only cooking conversation should be visible
      expect(screen.queryByText('Weather Questions')).not.toBeInTheDocument();
      expect(screen.getByText('Cooking Tips')).toBeInTheDocument();
    });

    it('shows no results message when search has no matches', async () => {
      const user = userEvent.setup();
      mockCommands.listAskAiConversations.mockResolvedValueOnce({
        status: 'ok',
        data: mockConversations,
      });

      render(<ConversationHistory />);

      await waitFor(() => {
        expect(screen.getByText('Weather Questions')).toBeInTheDocument();
      });

      // Search for something that doesn't exist
      const searchInput = screen.getByPlaceholderText('askAi.history.search');
      await user.type(searchInput, 'nonexistent query');

      expect(screen.getByText('askAi.history.noResults')).toBeInTheDocument();
    });
  });

  describe('refresh functionality', () => {
    it('reloads conversations when refresh button is clicked', async () => {
      const user = userEvent.setup();
      mockCommands.listAskAiConversations.mockResolvedValue({
        status: 'ok',
        data: [],
      });

      render(<ConversationHistory />);

      await waitFor(() => {
        expect(screen.getByText('askAi.history.empty')).toBeInTheDocument();
      });

      // Find refresh button and click it
      const refreshButton = screen.getAllByRole('button').find((btn) => {
        const svg = btn.querySelector('svg');
        return svg && !btn.className.includes('red');
      });

      expect(refreshButton).toBeDefined();
      if (refreshButton) {
        await user.click(refreshButton);
      }

      // Should have called listAskAiConversations again
      expect(mockCommands.listAskAiConversations).toHaveBeenCalledTimes(2);
    });
  });
});
