use std::cmp::{max, min};

use tui::layout::{Constraint, Direction, Layout, Rect};

type Len = u16;

type Ratio = f32;

#[derive(Clone)]
#[allow(dead_code)]
pub enum GridConstraint {
    Fixed(Len),
    Percent(Len),
    Fr(Ratio),
}

impl GridConstraint {
    pub fn to_constraints(total: Len, grid_constraints: &Vec<GridConstraint>) -> Vec<Constraint> {
        let n = grid_constraints.len();
        let mut remain = total;
        let mut constraints = Vec::with_capacity(n);
        let mut fr = false;
        let mut total_fr = 0f32;
        for c in grid_constraints {
            let len = match c {
                GridConstraint::Fixed(len) => *len,
                GridConstraint::Percent(pct) => {
                    let p = (((*pct as Ratio) * 1f32) * (total as Ratio)) as Len;
                    max(0, min(p, remain))
                }
                GridConstraint::Fr(r) => {
                    total_fr += *r;
                    fr = true;
                    0
                }
            };

            remain -= min(remain, len);
            constraints.push(Constraint::Length(len));
        }

        if fr {
            let fr_value = (remain as Ratio) / total_fr;
            for (c, gc) in constraints.iter_mut().zip(grid_constraints) {
                let GridConstraint::Fr(fr) = gc else { continue };
                let len = (*fr * fr_value) as Len;
                *c = Constraint::Length(max(1, len));
                remain -= len;
            }
        }

        while remain > 0 {
            let mut changed = false;
            for (c, gc) in constraints.iter_mut().zip(grid_constraints) {
                if remain == 0 {
                    break;
                }
                let (GridConstraint::Fr(..) | GridConstraint::Percent(..)) = gc else {
                    continue;
                };
                let Constraint::Length(len) = c else { continue };
                *len += 1;
                remain -= 1;
                changed = true;
            }

            if !changed {
                break;
            }
        }

        constraints
    }
}

#[derive(Default)]
pub struct GridLayout {
    _rect: Rect,
    _row: Vec<GridConstraint>,
    _col: Vec<GridConstraint>,
}

impl GridLayout {
    pub fn rect(&mut self, rect: Rect) -> &mut Self {
        self._rect = rect;
        self
    }

    pub fn row<T: Into<Vec<GridConstraint>>>(&mut self, row: T) -> &mut Self {
        self._row = row.into();
        self
    }

    pub fn col<T: Into<Vec<GridConstraint>>>(&mut self, col: T) -> &mut Self {
        self._col = col.into();
        self
    }

    pub fn split(&self) -> Vec<Vec<Rect>> {
        let mut v = vec![];
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(GridConstraint::to_constraints(self._rect.height, &self._col))
            .split(self._rect);

        for row in rows.iter() {
            v.push(
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(GridConstraint::to_constraints(row.width, &self._row))
                    .split(*row),
            )
        }
        v
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        assert_eq!(
            vec![Constraint::Length(10), Constraint::Length(50), Constraint::Length(40),],
            GridConstraint::to_constraints(
                100,
                &vec![
                    GridConstraint::Fr(1.0),
                    GridConstraint::Fixed(50),
                    GridConstraint::Fr(4.0),
                ]
            )
        )
    }
}
