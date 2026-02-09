import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ToggleSwitch } from './ToggleSwitch';

describe('ToggleSwitch', () => {
  const defaultProps = {
    checked: false,
    onChange: vi.fn(),
    label: 'Test Toggle',
    description: 'Test description for the toggle',
  };

  describe('rendering', () => {
    it('renders with label', () => {
      render(<ToggleSwitch {...defaultProps} />);
      expect(screen.getByText('Test Toggle')).toBeInTheDocument();
    });

    it('renders checkbox input', () => {
      render(<ToggleSwitch {...defaultProps} />);
      expect(screen.getByRole('checkbox')).toBeInTheDocument();
    });

    it('renders as unchecked when checked is false', () => {
      render(<ToggleSwitch {...defaultProps} checked={false} />);
      expect(screen.getByRole('checkbox')).not.toBeChecked();
    });

    it('renders as checked when checked is true', () => {
      render(<ToggleSwitch {...defaultProps} checked={true} />);
      expect(screen.getByRole('checkbox')).toBeChecked();
    });
  });

  describe('interactions', () => {
    it('calls onChange with true when toggling on', async () => {
      const user = userEvent.setup();
      const handleChange = vi.fn();
      render(<ToggleSwitch {...defaultProps} onChange={handleChange} checked={false} />);

      await user.click(screen.getByRole('checkbox'));
      expect(handleChange).toHaveBeenCalledWith(true);
    });

    it('calls onChange with false when toggling off', async () => {
      const user = userEvent.setup();
      const handleChange = vi.fn();
      render(<ToggleSwitch {...defaultProps} onChange={handleChange} checked={true} />);

      await user.click(screen.getByRole('checkbox'));
      expect(handleChange).toHaveBeenCalledWith(false);
    });

    it('does not call onChange when disabled', async () => {
      const user = userEvent.setup();
      const handleChange = vi.fn();
      render(<ToggleSwitch {...defaultProps} onChange={handleChange} disabled={true} />);

      await user.click(screen.getByRole('checkbox'));
      expect(handleChange).not.toHaveBeenCalled();
    });

    it('does not call onChange when isUpdating', async () => {
      const user = userEvent.setup();
      const handleChange = vi.fn();
      render(<ToggleSwitch {...defaultProps} onChange={handleChange} isUpdating={true} />);

      await user.click(screen.getByRole('checkbox'));
      expect(handleChange).not.toHaveBeenCalled();
    });
  });

  describe('disabled state', () => {
    it('checkbox is disabled when disabled prop is true', () => {
      render(<ToggleSwitch {...defaultProps} disabled={true} />);
      expect(screen.getByRole('checkbox')).toBeDisabled();
    });

    it('checkbox is disabled when isUpdating prop is true', () => {
      render(<ToggleSwitch {...defaultProps} isUpdating={true} />);
      expect(screen.getByRole('checkbox')).toBeDisabled();
    });

    it('shows loading spinner when isUpdating', () => {
      render(<ToggleSwitch {...defaultProps} isUpdating={true} />);
      // The spinner has animate-spin class
      expect(document.querySelector('.animate-spin')).toBeInTheDocument();
    });
  });

  describe('description modes', () => {
    it('renders with tooltip description mode by default', () => {
      render(<ToggleSwitch {...defaultProps} />);
      // In tooltip mode, description is shown via info icon hover
      expect(screen.getByRole('button', { name: 'More information' })).toBeInTheDocument();
    });

    it('renders with inline description mode', () => {
      render(<ToggleSwitch {...defaultProps} descriptionMode="inline" />);
      expect(screen.getByText('Test description for the toggle')).toBeInTheDocument();
    });
  });
});
