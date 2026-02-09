import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Dropdown, DropdownOption } from './Dropdown';

const mockOptions: DropdownOption[] = [
  { value: 'option1', label: 'Option 1' },
  { value: 'option2', label: 'Option 2' },
  { value: 'option3', label: 'Option 3' },
];

describe('Dropdown', () => {
  describe('rendering', () => {
    it('renders with placeholder when no value selected', () => {
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );
      expect(screen.getByRole('combobox')).toHaveTextContent('Select an option...');
    });

    it('renders with custom placeholder', () => {
      render(
        <Dropdown
          options={mockOptions}
          selectedValue={null}
          onSelect={vi.fn()}
          placeholder="Choose..."
        />
      );
      expect(screen.getByRole('combobox')).toHaveTextContent('Choose...');
    });

    it('renders selected option label', () => {
      render(
        <Dropdown options={mockOptions} selectedValue="option2" onSelect={vi.fn()} />
      );
      expect(screen.getByRole('combobox')).toHaveTextContent('Option 2');
    });

    it('does not show listbox when closed', () => {
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );
      expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
    });
  });

  describe('opening and closing', () => {
    it('opens listbox when clicked', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );

      await user.click(screen.getByRole('combobox'));
      expect(screen.getByRole('listbox')).toBeInTheDocument();
    });

    it('closes listbox when clicking again', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );

      await user.click(screen.getByRole('combobox'));
      expect(screen.getByRole('listbox')).toBeInTheDocument();

      await user.click(screen.getByRole('combobox'));
      expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
    });

    it('closes listbox when pressing Escape', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );

      await user.click(screen.getByRole('combobox'));
      expect(screen.getByRole('listbox')).toBeInTheDocument();

      await user.keyboard('{Escape}');
      expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
    });
  });

  describe('selection', () => {
    it('calls onSelect when option is clicked', async () => {
      const user = userEvent.setup();
      const handleSelect = vi.fn();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={handleSelect} />
      );

      await user.click(screen.getByRole('combobox'));
      await user.click(screen.getByRole('option', { name: 'Option 2' }));

      expect(handleSelect).toHaveBeenCalledWith('option2');
    });

    it('closes listbox after selection', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );

      await user.click(screen.getByRole('combobox'));
      await user.click(screen.getByRole('option', { name: 'Option 1' }));

      expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
    });
  });

  describe('keyboard navigation', () => {
    it('opens listbox with Enter key', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );

      screen.getByRole('combobox').focus();
      await user.keyboard('{Enter}');

      expect(screen.getByRole('listbox')).toBeInTheDocument();
    });

    it('opens listbox with Space key', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );

      screen.getByRole('combobox').focus();
      await user.keyboard(' ');

      expect(screen.getByRole('listbox')).toBeInTheDocument();
    });

    it('opens listbox with ArrowDown key', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );

      screen.getByRole('combobox').focus();
      await user.keyboard('{ArrowDown}');

      expect(screen.getByRole('listbox')).toBeInTheDocument();
    });

    it('navigates options with arrow keys', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );

      await user.click(screen.getByRole('combobox'));

      // First option should be focused initially
      const combobox = screen.getByRole('combobox');
      expect(combobox).toHaveAttribute(
        'aria-activedescendant',
        expect.stringContaining('option-0')
      );

      // Navigate down
      await user.keyboard('{ArrowDown}');
      expect(combobox).toHaveAttribute(
        'aria-activedescendant',
        expect.stringContaining('option-1')
      );

      // Navigate up
      await user.keyboard('{ArrowUp}');
      expect(combobox).toHaveAttribute(
        'aria-activedescendant',
        expect.stringContaining('option-0')
      );
    });

    it('selects focused option with Enter', async () => {
      const user = userEvent.setup();
      const handleSelect = vi.fn();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={handleSelect} />
      );

      await user.click(screen.getByRole('combobox'));
      await user.keyboard('{ArrowDown}'); // Move to second option
      await user.keyboard('{Enter}');

      expect(handleSelect).toHaveBeenCalledWith('option2');
    });

    it('wraps around when navigating past last option', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );

      await user.click(screen.getByRole('combobox'));
      await user.keyboard('{ArrowDown}'); // option 2
      await user.keyboard('{ArrowDown}'); // option 3
      await user.keyboard('{ArrowDown}'); // wrap to option 1

      const combobox = screen.getByRole('combobox');
      expect(combobox).toHaveAttribute(
        'aria-activedescendant',
        expect.stringContaining('option-0')
      );
    });

    it('navigates to first option with Home key', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );

      await user.click(screen.getByRole('combobox'));
      await user.keyboard('{ArrowDown}');
      await user.keyboard('{ArrowDown}');
      await user.keyboard('{Home}');

      const combobox = screen.getByRole('combobox');
      expect(combobox).toHaveAttribute(
        'aria-activedescendant',
        expect.stringContaining('option-0')
      );
    });

    it('navigates to last option with End key', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );

      await user.click(screen.getByRole('combobox'));
      await user.keyboard('{End}');

      const combobox = screen.getByRole('combobox');
      expect(combobox).toHaveAttribute(
        'aria-activedescendant',
        expect.stringContaining('option-2')
      );
    });
  });

  describe('disabled state', () => {
    it('does not open when disabled', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown
          options={mockOptions}
          selectedValue={null}
          onSelect={vi.fn()}
          disabled
        />
      );

      await user.click(screen.getByRole('combobox'));
      expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
    });

    it('has disabled attribute when disabled', () => {
      render(
        <Dropdown
          options={mockOptions}
          selectedValue={null}
          onSelect={vi.fn()}
          disabled
        />
      );
      expect(screen.getByRole('combobox')).toBeDisabled();
    });
  });

  describe('accessibility', () => {
    it('has correct ARIA attributes when closed', () => {
      render(
        <Dropdown
          options={mockOptions}
          selectedValue={null}
          onSelect={vi.fn()}
          aria-label="Test dropdown"
        />
      );

      const combobox = screen.getByRole('combobox');
      expect(combobox).toHaveAttribute('aria-expanded', 'false');
      expect(combobox).toHaveAttribute('aria-haspopup', 'listbox');
      expect(combobox).toHaveAttribute('aria-label', 'Test dropdown');
    });

    it('has correct ARIA attributes when open', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue={null} onSelect={vi.fn()} />
      );

      await user.click(screen.getByRole('combobox'));

      expect(screen.getByRole('combobox')).toHaveAttribute('aria-expanded', 'true');
      expect(screen.getByRole('listbox')).toBeInTheDocument();
    });

    it('marks selected option with aria-selected', async () => {
      const user = userEvent.setup();
      render(
        <Dropdown options={mockOptions} selectedValue="option2" onSelect={vi.fn()} />
      );

      await user.click(screen.getByRole('combobox'));

      const options = screen.getAllByRole('option');
      expect(options[0]).toHaveAttribute('aria-selected', 'false');
      expect(options[1]).toHaveAttribute('aria-selected', 'true');
      expect(options[2]).toHaveAttribute('aria-selected', 'false');
    });
  });

  describe('empty state', () => {
    it('shows no options message when options array is empty', async () => {
      const user = userEvent.setup();
      render(<Dropdown options={[]} selectedValue={null} onSelect={vi.fn()} />);

      await user.click(screen.getByRole('combobox'));

      expect(screen.getByText('common.noOptionsFound')).toBeInTheDocument();
    });
  });

  describe('refresh callback', () => {
    it('calls onRefresh when opening dropdown', async () => {
      const user = userEvent.setup();
      const handleRefresh = vi.fn();
      render(
        <Dropdown
          options={mockOptions}
          selectedValue={null}
          onSelect={vi.fn()}
          onRefresh={handleRefresh}
        />
      );

      await user.click(screen.getByRole('combobox'));

      expect(handleRefresh).toHaveBeenCalledTimes(1);
    });
  });

  describe('disabled options', () => {
    it('skips disabled options in keyboard navigation', async () => {
      const user = userEvent.setup();
      const optionsWithDisabled: DropdownOption[] = [
        { value: 'option1', label: 'Option 1' },
        { value: 'option2', label: 'Option 2', disabled: true },
        { value: 'option3', label: 'Option 3' },
      ];

      render(
        <Dropdown
          options={optionsWithDisabled}
          selectedValue={null}
          onSelect={vi.fn()}
        />
      );

      await user.click(screen.getByRole('combobox'));

      // Should only show enabled options (Option 1 and Option 3)
      const options = screen.getAllByRole('option');
      expect(options).toHaveLength(2);
    });
  });
});
