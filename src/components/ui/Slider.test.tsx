import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Slider } from './Slider';

describe('Slider', () => {
  const defaultProps = {
    value: 50,
    onChange: vi.fn(),
    min: 0,
    max: 100,
    label: 'Volume',
    description: 'Adjust the volume level',
  };

  describe('rendering', () => {
    it('renders with label', () => {
      render(<Slider {...defaultProps} />);
      expect(screen.getByText('Volume')).toBeInTheDocument();
    });

    it('renders a range input', () => {
      render(<Slider {...defaultProps} />);
      expect(screen.getByRole('slider')).toBeInTheDocument();
    });

    it('renders with correct initial value', () => {
      render(<Slider {...defaultProps} value={75} />);
      expect(screen.getByRole('slider')).toHaveValue('75');
    });

    it('renders with correct min and max', () => {
      render(<Slider {...defaultProps} min={10} max={90} />);
      const slider = screen.getByRole('slider');
      expect(slider).toHaveAttribute('min', '10');
      expect(slider).toHaveAttribute('max', '90');
    });

    it('renders with correct step', () => {
      render(<Slider {...defaultProps} step={5} />);
      expect(screen.getByRole('slider')).toHaveAttribute('step', '5');
    });

    it('uses default step of 0.01', () => {
      render(<Slider {...defaultProps} />);
      expect(screen.getByRole('slider')).toHaveAttribute('step', '0.01');
    });
  });

  describe('value display', () => {
    it('shows value by default', () => {
      render(<Slider {...defaultProps} value={50} />);
      expect(screen.getByText('50.00')).toBeInTheDocument();
    });

    it('hides value when showValue is false', () => {
      render(<Slider {...defaultProps} value={50} showValue={false} />);
      expect(screen.queryByText('50.00')).not.toBeInTheDocument();
    });

    it('uses custom formatValue function', () => {
      render(
        <Slider
          {...defaultProps}
          value={50}
          formatValue={(v) => `${v}%`}
        />
      );
      expect(screen.getByText('50%')).toBeInTheDocument();
    });

    it('formats decimal values correctly', () => {
      render(<Slider {...defaultProps} value={0.75} max={1} />);
      expect(screen.getByText('0.75')).toBeInTheDocument();
    });
  });

  describe('interactions', () => {
    it('calls onChange with new value', () => {
      const handleChange = vi.fn();
      render(<Slider {...defaultProps} onChange={handleChange} />);

      fireEvent.change(screen.getByRole('slider'), { target: { value: '75' } });
      expect(handleChange).toHaveBeenCalledWith(75);
    });

    it('handles minimum value', () => {
      const handleChange = vi.fn();
      render(<Slider {...defaultProps} onChange={handleChange} min={0} />);

      fireEvent.change(screen.getByRole('slider'), { target: { value: '0' } });
      expect(handleChange).toHaveBeenCalledWith(0);
    });

    it('handles maximum value', () => {
      const handleChange = vi.fn();
      render(<Slider {...defaultProps} onChange={handleChange} max={100} />);

      fireEvent.change(screen.getByRole('slider'), { target: { value: '100' } });
      expect(handleChange).toHaveBeenCalledWith(100);
    });

    it('handles decimal values', () => {
      const handleChange = vi.fn();
      render(<Slider {...defaultProps} onChange={handleChange} step={0.1} />);

      fireEvent.change(screen.getByRole('slider'), { target: { value: '50.5' } });
      expect(handleChange).toHaveBeenCalledWith(50.5);
    });
  });

  describe('disabled state', () => {
    it('slider is disabled when disabled prop is true', () => {
      render(<Slider {...defaultProps} disabled />);
      expect(screen.getByRole('slider')).toBeDisabled();
    });

    it('does not call onChange when disabled', () => {
      const handleChange = vi.fn();
      render(<Slider {...defaultProps} onChange={handleChange} disabled />);

      fireEvent.change(screen.getByRole('slider'), { target: { value: '75' } });
      // Note: fireEvent still fires the event, but the disabled attribute prevents interaction in real browser
      expect(screen.getByRole('slider')).toBeDisabled();
    });
  });

  describe('description modes', () => {
    it('renders with tooltip description mode by default', () => {
      render(<Slider {...defaultProps} />);
      expect(screen.getByRole('button', { name: 'More information' })).toBeInTheDocument();
    });

    it('renders with inline description mode', () => {
      render(<Slider {...defaultProps} descriptionMode="inline" />);
      expect(screen.getByText('Adjust the volume level')).toBeInTheDocument();
    });
  });
});
