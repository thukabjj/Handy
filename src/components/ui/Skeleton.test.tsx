import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Skeleton, SkeletonCard, SkeletonSettingRow, SkeletonModelCard } from './Skeleton';

describe('Skeleton', () => {
  describe('basic rendering', () => {
    it('renders with default props', () => {
      render(<Skeleton />);
      const skeleton = document.querySelector('[aria-hidden="true"]');
      expect(skeleton).toBeInTheDocument();
    });

    it('has aria-hidden attribute for accessibility', () => {
      render(<Skeleton />);
      const skeleton = document.querySelector('[aria-hidden="true"]');
      expect(skeleton).toHaveAttribute('aria-hidden', 'true');
    });

    it('has presentation role', () => {
      render(<Skeleton />);
      // Elements with aria-hidden="true" require hidden: true option
      expect(screen.getByRole('presentation', { hidden: true })).toBeInTheDocument();
    });
  });

  describe('dimensions', () => {
    it('applies default width (w-full)', () => {
      render(<Skeleton />);
      const skeleton = document.querySelector('[aria-hidden="true"]');
      expect(skeleton).toHaveClass('w-full');
    });

    it('applies default height (h-4)', () => {
      render(<Skeleton />);
      const skeleton = document.querySelector('[aria-hidden="true"]');
      expect(skeleton).toHaveClass('h-4');
    });

    it('applies custom width', () => {
      render(<Skeleton width="w-32" />);
      const skeleton = document.querySelector('[aria-hidden="true"]');
      expect(skeleton).toHaveClass('w-32');
    });

    it('applies custom height', () => {
      render(<Skeleton height="h-10" />);
      const skeleton = document.querySelector('[aria-hidden="true"]');
      expect(skeleton).toHaveClass('h-10');
    });
  });

  describe('shapes', () => {
    it('renders with rounded corners by default', () => {
      render(<Skeleton />);
      const skeleton = document.querySelector('[aria-hidden="true"]');
      expect(skeleton).toHaveClass('rounded');
    });

    it('renders as circle when circle prop is true', () => {
      render(<Skeleton circle />);
      const skeleton = document.querySelector('[aria-hidden="true"]');
      expect(skeleton).toHaveClass('rounded-full');
    });
  });

  describe('animations', () => {
    it('applies pulse animation by default', () => {
      render(<Skeleton />);
      const skeleton = document.querySelector('[aria-hidden="true"]');
      expect(skeleton).toHaveClass('animate-pulse');
    });

    it('applies shimmer animation when specified', () => {
      render(<Skeleton animation="shimmer" />);
      const skeleton = document.querySelector('[aria-hidden="true"]');
      expect(skeleton).toHaveClass('overflow-hidden');
    });

    it('applies no animation when animation is none', () => {
      render(<Skeleton animation="none" />);
      const skeleton = document.querySelector('[aria-hidden="true"]');
      expect(skeleton).not.toHaveClass('animate-pulse');
    });
  });

  describe('custom className', () => {
    it('applies additional className', () => {
      render(<Skeleton className="my-custom-class" />);
      const skeleton = document.querySelector('[aria-hidden="true"]');
      expect(skeleton).toHaveClass('my-custom-class');
    });
  });
});

describe('SkeletonCard', () => {
  it('renders multiple skeleton lines', () => {
    render(<SkeletonCard />);
    const skeletons = document.querySelectorAll('[aria-hidden="true"]');
    expect(skeletons.length).toBeGreaterThan(1);
  });

  it('applies custom className', () => {
    render(<SkeletonCard className="custom-card" />);
    const card = document.querySelector('.custom-card');
    expect(card).toBeInTheDocument();
  });

  it('has card-like styling', () => {
    render(<SkeletonCard />);
    const card = document.querySelector('.rounded-card');
    expect(card).toBeInTheDocument();
  });
});

describe('SkeletonSettingRow', () => {
  it('renders with presentation role', () => {
    render(<SkeletonSettingRow />);
    // Elements with aria-hidden="true" require hidden: true option
    expect(screen.getAllByRole('presentation', { hidden: true }).length).toBeGreaterThan(0);
  });

  it('renders multiple skeletons for label and control', () => {
    render(<SkeletonSettingRow />);
    const skeletons = document.querySelectorAll('.bg-mid-gray\\/20');
    expect(skeletons.length).toBeGreaterThanOrEqual(3);
  });

  it('applies custom className', () => {
    render(<SkeletonSettingRow className="custom-row" />);
    const row = document.querySelector('.custom-row');
    expect(row).toBeInTheDocument();
  });
});

describe('SkeletonModelCard', () => {
  it('renders with presentation role', () => {
    render(<SkeletonModelCard />);
    // Elements with aria-hidden="true" require hidden: true option
    expect(screen.getAllByRole('presentation', { hidden: true }).length).toBeGreaterThan(0);
  });

  it('renders a circular skeleton for avatar', () => {
    render(<SkeletonModelCard />);
    const circles = document.querySelectorAll('.rounded-full');
    expect(circles.length).toBeGreaterThanOrEqual(1);
  });

  it('applies custom className', () => {
    render(<SkeletonModelCard className="custom-model" />);
    const card = document.querySelector('.custom-model');
    expect(card).toBeInTheDocument();
  });
});
