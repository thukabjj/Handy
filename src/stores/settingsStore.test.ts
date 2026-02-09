import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { act } from '@testing-library/react';
import { useSettingsStore } from './settingsStore';
import { commands } from '@/bindings';

// Get the mocked commands module
const mockCommands = vi.mocked(commands);

describe('settingsStore', () => {
  beforeEach(() => {
    // Reset store state before each test
    const { getState, setState } = useSettingsStore;
    setState({
      settings: null,
      defaultSettings: null,
      isLoading: true,
      isUpdating: {},
      audioDevices: [],
      outputDevices: [],
      customSounds: { start: false, stop: false },
      postProcessModelOptions: {},
    });

    // Clear all mock calls
    vi.clearAllMocks();
  });

  describe('initial state', () => {
    it('has correct initial values', () => {
      const state = useSettingsStore.getState();
      expect(state.settings).toBeNull();
      expect(state.defaultSettings).toBeNull();
      expect(state.isLoading).toBe(true);
      expect(state.audioDevices).toEqual([]);
      expect(state.outputDevices).toEqual([]);
    });
  });

  describe('refreshSettings', () => {
    it('loads settings successfully', async () => {
      const mockSettings = {
        push_to_talk: true,
        bindings: {},
        audio_feedback: true,
      };

      mockCommands.getAppSettings.mockResolvedValueOnce({
        status: 'ok',
        data: mockSettings as any,
      });

      await act(async () => {
        await useSettingsStore.getState().refreshSettings();
      });

      const state = useSettingsStore.getState();
      expect(state.isLoading).toBe(false);
      expect(state.settings).toBeDefined();
      expect(state.settings?.push_to_talk).toBe(true);
    });

    it('handles error gracefully', async () => {
      mockCommands.getAppSettings.mockResolvedValueOnce({
        status: 'error',
        error: 'Failed to load',
      });

      await act(async () => {
        await useSettingsStore.getState().refreshSettings();
      });

      const state = useSettingsStore.getState();
      expect(state.isLoading).toBe(false);
      expect(state.settings).toBeNull();
    });

    it('normalizes microphone settings with defaults', async () => {
      const mockSettings = {
        push_to_talk: true,
        bindings: {},
        audio_feedback: true,
        always_on_microphone: undefined,
        selected_microphone: undefined,
        clamshell_microphone: undefined,
        selected_output_device: undefined,
      };

      mockCommands.getAppSettings.mockResolvedValueOnce({
        status: 'ok',
        data: mockSettings as any,
      });

      await act(async () => {
        await useSettingsStore.getState().refreshSettings();
      });

      const state = useSettingsStore.getState();
      expect(state.settings?.always_on_microphone).toBe(false);
      expect(state.settings?.selected_microphone).toBe('Default');
      expect(state.settings?.clamshell_microphone).toBe('Default');
      expect(state.settings?.selected_output_device).toBe('Default');
    });
  });

  describe('refreshAudioDevices', () => {
    it('loads audio devices and prepends default', async () => {
      mockCommands.getAvailableMicrophones.mockResolvedValueOnce({
        status: 'ok',
        data: [
          { index: '1', name: 'Microphone 1', is_default: false },
          { index: '2', name: 'Microphone 2', is_default: true },
        ],
      });

      await act(async () => {
        await useSettingsStore.getState().refreshAudioDevices();
      });

      const state = useSettingsStore.getState();
      expect(state.audioDevices).toHaveLength(3);
      expect(state.audioDevices[0]).toEqual({
        index: 'default',
        name: 'Default',
        is_default: true,
      });
    });

    it('filters out devices named "Default" or "default"', async () => {
      mockCommands.getAvailableMicrophones.mockResolvedValueOnce({
        status: 'ok',
        data: [
          { index: '1', name: 'Default', is_default: true },
          { index: '2', name: 'Microphone 1', is_default: false },
        ],
      });

      await act(async () => {
        await useSettingsStore.getState().refreshAudioDevices();
      });

      const state = useSettingsStore.getState();
      expect(state.audioDevices).toHaveLength(2);
      expect(state.audioDevices.find((d) => d.name === 'Default' && d.index === '1')).toBeUndefined();
    });

    it('handles error by setting default device only', async () => {
      mockCommands.getAvailableMicrophones.mockResolvedValueOnce({
        status: 'error',
        error: 'Failed to load devices',
      });

      await act(async () => {
        await useSettingsStore.getState().refreshAudioDevices();
      });

      const state = useSettingsStore.getState();
      expect(state.audioDevices).toHaveLength(1);
      expect(state.audioDevices[0].name).toBe('Default');
    });
  });

  describe('refreshOutputDevices', () => {
    it('loads output devices and prepends default', async () => {
      mockCommands.getAvailableOutputDevices.mockResolvedValueOnce({
        status: 'ok',
        data: [
          { index: '1', name: 'Speaker 1', is_default: false },
          { index: '2', name: 'Speaker 2', is_default: true },
        ],
      });

      await act(async () => {
        await useSettingsStore.getState().refreshOutputDevices();
      });

      const state = useSettingsStore.getState();
      expect(state.outputDevices).toHaveLength(3);
      expect(state.outputDevices[0].name).toBe('Default');
    });
  });

  describe('updateSetting', () => {
    beforeEach(async () => {
      // Set up initial settings
      useSettingsStore.setState({
        settings: {
          push_to_talk: true,
          bindings: {},
          audio_feedback: true,
        } as any,
        isLoading: false,
      });
    });

    it('updates setting optimistically', async () => {
      mockCommands.changePttSetting.mockResolvedValueOnce({
        status: 'ok',
        data: null,
      });

      await act(async () => {
        await useSettingsStore.getState().updateSetting('push_to_talk', false);
      });

      expect(useSettingsStore.getState().settings?.push_to_talk).toBe(false);
      expect(mockCommands.changePttSetting).toHaveBeenCalledWith(false);
    });

    it('sets updating state during update', async () => {
      let updatingDuringCall = false;

      mockCommands.changeAudioFeedbackSetting.mockImplementation(async () => {
        updatingDuringCall = useSettingsStore.getState().isUpdatingKey('audio_feedback');
        return { status: 'ok', data: null };
      });

      await act(async () => {
        await useSettingsStore.getState().updateSetting('audio_feedback', false);
      });

      expect(updatingDuringCall).toBe(true);
      expect(useSettingsStore.getState().isUpdatingKey('audio_feedback')).toBe(false);
    });

    it('rolls back on error', async () => {
      mockCommands.changePttSetting.mockRejectedValueOnce(new Error('Failed'));

      await act(async () => {
        try {
          await useSettingsStore.getState().updateSetting('push_to_talk', false);
        } catch {
          // Expected to throw
        }
      });

      expect(useSettingsStore.getState().settings?.push_to_talk).toBe(true);
    });
  });

  describe('updateBinding', () => {
    beforeEach(() => {
      useSettingsStore.setState({
        settings: {
          push_to_talk: true,
          bindings: {
            shortcut: {
              id: 'shortcut',
              name: 'Test Shortcut',
              description: 'A test shortcut',
              default_binding: 'Ctrl+A',
              current_binding: 'Ctrl+A',
            },
          },
          audio_feedback: true,
        } as any,
        isLoading: false,
      });
    });

    it('updates binding optimistically', async () => {
      mockCommands.changeBinding.mockResolvedValueOnce({
        status: 'ok',
        data: { success: true, binding: null, error: null },
      });

      await act(async () => {
        await useSettingsStore.getState().updateBinding('shortcut', 'Ctrl+B');
      });

      const binding = useSettingsStore.getState().settings?.bindings?.shortcut;
      expect(binding?.current_binding).toBe('Ctrl+B');
    });

    it('rolls back binding on failure', async () => {
      mockCommands.changeBinding.mockResolvedValueOnce({
        status: 'ok',
        data: { success: false, binding: null, error: 'Invalid binding' },
      });

      await act(async () => {
        try {
          await useSettingsStore.getState().updateBinding('shortcut', 'Invalid');
        } catch {
          // Expected
        }
      });

      const binding = useSettingsStore.getState().settings?.bindings?.shortcut;
      expect(binding?.current_binding).toBe('Ctrl+A');
    });
  });

  describe('resetBinding', () => {
    beforeEach(() => {
      useSettingsStore.setState({
        settings: {
          push_to_talk: true,
          bindings: {
            shortcut: {
              id: 'shortcut',
              name: 'Test Shortcut',
              description: 'A test shortcut',
              default_binding: 'Ctrl+A',
              current_binding: 'Ctrl+B',
            },
          },
          audio_feedback: true,
        } as any,
        isLoading: false,
      });
    });

    it('calls resetBinding command and refreshes', async () => {
      mockCommands.resetBinding.mockResolvedValueOnce({
        status: 'ok',
        data: { success: true, binding: null, error: null },
      });
      mockCommands.getAppSettings.mockResolvedValueOnce({
        status: 'ok',
        data: {
          push_to_talk: true,
          bindings: {
            shortcut: {
              id: 'shortcut',
              name: 'Test Shortcut',
              description: 'A test shortcut',
              default_binding: 'Ctrl+A',
              current_binding: 'Ctrl+A',
            },
          },
          audio_feedback: true,
        } as any,
      });

      await act(async () => {
        await useSettingsStore.getState().resetBinding('shortcut');
      });

      expect(mockCommands.resetBinding).toHaveBeenCalledWith('shortcut');
      expect(mockCommands.getAppSettings).toHaveBeenCalled();
    });
  });

  describe('getSetting', () => {
    it('returns setting value when settings exist', () => {
      useSettingsStore.setState({
        settings: {
          push_to_talk: true,
          bindings: {},
          audio_feedback: false,
        } as any,
      });

      expect(useSettingsStore.getState().getSetting('push_to_talk')).toBe(true);
      expect(useSettingsStore.getState().getSetting('audio_feedback')).toBe(false);
    });

    it('returns undefined when settings is null', () => {
      useSettingsStore.setState({ settings: null });
      expect(useSettingsStore.getState().getSetting('push_to_talk')).toBeUndefined();
    });
  });

  describe('isUpdatingKey', () => {
    it('returns true when key is being updated', () => {
      useSettingsStore.setState({
        isUpdating: { push_to_talk: true },
      });

      expect(useSettingsStore.getState().isUpdatingKey('push_to_talk')).toBe(true);
    });

    it('returns false when key is not being updated', () => {
      useSettingsStore.setState({
        isUpdating: {},
      });

      expect(useSettingsStore.getState().isUpdatingKey('push_to_talk')).toBe(false);
    });
  });

  describe('post-process settings', () => {
    beforeEach(() => {
      useSettingsStore.setState({
        settings: {
          push_to_talk: true,
          bindings: {},
          audio_feedback: true,
          post_process_provider_id: 'openai',
        } as any,
        isLoading: false,
      });
    });

    it('setPostProcessProvider updates provider and refreshes', async () => {
      mockCommands.setPostProcessProvider.mockResolvedValueOnce({
        status: 'ok',
        data: null,
      });
      mockCommands.getAppSettings.mockResolvedValueOnce({
        status: 'ok',
        data: {
          push_to_talk: true,
          bindings: {},
          audio_feedback: true,
          post_process_provider_id: 'anthropic',
        } as any,
      });

      await act(async () => {
        await useSettingsStore.getState().setPostProcessProvider('anthropic');
      });

      expect(mockCommands.setPostProcessProvider).toHaveBeenCalledWith('anthropic');
    });

    it('fetchPostProcessModels returns models from backend', async () => {
      mockCommands.fetchPostProcessModels.mockResolvedValueOnce({
        status: 'ok',
        data: ['gpt-4', 'gpt-3.5-turbo'],
      });

      let models: string[] = [];
      await act(async () => {
        models = await useSettingsStore.getState().fetchPostProcessModels('openai');
      });

      expect(models).toEqual(['gpt-4', 'gpt-3.5-turbo']);
      expect(useSettingsStore.getState().postProcessModelOptions['openai']).toEqual([
        'gpt-4',
        'gpt-3.5-turbo',
      ]);
    });

    it('updatePostProcessApiKey clears cached models', async () => {
      useSettingsStore.setState({
        postProcessModelOptions: { openai: ['gpt-4', 'gpt-3.5-turbo'] },
      });

      mockCommands.changePostProcessApiKeySetting.mockResolvedValueOnce({
        status: 'ok',
        data: null,
      });
      mockCommands.getAppSettings.mockResolvedValueOnce({
        status: 'ok',
        data: {
          push_to_talk: true,
          bindings: {},
          audio_feedback: true,
        } as any,
      });

      await act(async () => {
        await useSettingsStore.getState().updatePostProcessApiKey('openai', 'new-key');
      });

      expect(useSettingsStore.getState().postProcessModelOptions['openai']).toEqual([]);
    });
  });

  describe('initialize', () => {
    it('loads default settings, current settings, and checks custom sounds', async () => {
      mockCommands.getDefaultSettings.mockResolvedValueOnce({
        status: 'ok',
        data: { push_to_talk: true, bindings: {}, audio_feedback: true } as any,
      });
      mockCommands.getAppSettings.mockResolvedValueOnce({
        status: 'ok',
        data: { push_to_talk: false, bindings: {}, audio_feedback: true } as any,
      });
      mockCommands.checkCustomSounds.mockResolvedValueOnce({ start: true, stop: false });

      await act(async () => {
        await useSettingsStore.getState().initialize();
      });

      const state = useSettingsStore.getState();
      expect(state.defaultSettings).toBeDefined();
      expect(state.settings).toBeDefined();
      expect(state.customSounds).toEqual({ start: true, stop: false });
    });
  });
});
