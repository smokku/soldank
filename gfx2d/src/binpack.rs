#[derive(Copy, Clone)]
pub struct Rect<T> {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub data: T,
}

impl<T> Rect<T> {
    pub fn left(&self)   -> i32 { self.x }
    pub fn right(&self)  -> i32 { self.x + self.w }
    pub fn top(&self)    -> i32 { self.y }
    pub fn bottom(&self) -> i32 { self.y + self.h }

    pub fn contains(&self, rc: &Rect<T>) -> bool {
        rc.left() >= self.left() && rc.top() >= self.top() &&
        rc.right() <= self.right() && rc.bottom() <= self.bottom()
    }
}

type BPRect = Rect<()>;

struct BinPack {
    used: Vec<BPRect>,
    free: Vec<BPRect>,
}

const MAX_SCORE: (i32, i32) = (::std::i32::MAX, ::std::i32::MAX);

pub fn pack_rects<T>(width: i32, height: i32, rects: &mut [Rect<T>]) -> usize {
    let mut bp = BinPack {
        used: vec![],
        free: vec![BPRect{x: 0, y: 0, w: width, h: height, data: ()}],
    };

    for i in 0..rects.len() {
        let mut best_index = rects.len();
        let mut best_rect = BPRect{x: 0, y: 0, w: 0, h: 0, data: ()};
        let mut best_score = MAX_SCORE;

        for j in i..rects.len() {
            let (rc, score) = score_rect(&bp, rects[j].w, rects[j].h);

            if score.0 < best_score.0 || (score.0 == best_score.0 && score.1 < best_score.1) {
                best_index = j;
                best_score = score;
                best_rect = rc;
            }
        }

        if best_index < rects.len() && best_rect.h != 0 {
            place_rect(&mut bp, &best_rect);

            rects[best_index].x = best_rect.x;
            rects[best_index].y = best_rect.y;
            rects[best_index].w = best_rect.w;
            rects[best_index].h = best_rect.h;

            rects.swap(i, best_index);
        } else {
            return i;
        }
    }

    rects.len()
}

fn score_rect(bp: &BinPack, w: i32, h: i32) -> (BPRect, (i32, i32)) {
    let mut best_rect = BPRect{x: 0, y: 0, w: 0, h: 0, data: ()};
    let mut best_score = MAX_SCORE;

    for free in &bp.free {
        if free.w >= w && free.h >= h {
            let top_side_y = free.y + h;

            if top_side_y < best_score.0 || (top_side_y == best_score.0 && free.x < best_score.1) {
                best_rect.x = free.x;
                best_rect.y = free.y;
                best_rect.w = w;
                best_rect.h = h;
                best_score = (top_side_y, free.x);
            }
        }
    }

    (best_rect, if best_rect.h == 0 { MAX_SCORE } else { best_score })
}

fn place_rect(bp: &mut BinPack, rect: &BPRect) {
    let mut i = 0usize;

    while i < bp.free.len() {
        let free_rect = bp.free[i];

        if split_free_rect(bp, &free_rect, &rect) {
            bp.free.remove(i);
        }
        else {
            i += 1;
        }
    }

    prune_free_list(bp);
    bp.used.push(*rect);
}

fn split_free_rect(bp: &mut BinPack, free_rect: &BPRect, used_rect: &BPRect) -> bool {
    if used_rect.left() >= free_rect.right() || used_rect.right() <= free_rect.left() ||
        used_rect.top() >= free_rect.bottom() || used_rect.bottom() <= free_rect.top() {
        return false;
    }

    if used_rect.left() < free_rect.right() && used_rect.right() > free_rect.left() {
        if used_rect.top() > free_rect.top() && used_rect.top() < free_rect.bottom() {
            bp.free.push(BPRect{
                h: used_rect.y - free_rect.y,
                ..*free_rect
            });
        }

        if used_rect.bottom() < free_rect.bottom() {
            bp.free.push(BPRect{
                y: used_rect.bottom(),
                h: free_rect.bottom() - used_rect.bottom(),
                ..*free_rect
            });
        }
    }

    if used_rect.top() < free_rect.bottom() && used_rect.bottom() > free_rect.top() {
        if used_rect.left() > free_rect.left() && used_rect.left() < free_rect.right() {
            bp.free.push(BPRect{
                w: used_rect.x - free_rect.x,
                ..*free_rect
            });
        }

        if used_rect.right() < free_rect.right() {
            bp.free.push(BPRect{
                x: used_rect.right(),
                w: free_rect.right() - used_rect.right(),
                ..*free_rect
            });
        }
    }

    true
}

fn prune_free_list(bp: &mut BinPack) {
    let mut i = 0;

    while i < bp.free.len() {
        let mut j = i + 1;

        while j < bp.free.len() {
            if bp.free[j].contains(&bp.free[i]) {
                bp.free.remove(i);
                i = i.wrapping_sub(1);
                break;
            } else if bp.free[i].contains(&bp.free[j]) {
                bp.free.remove(j);
            }
            else {
                j += 1;
            }
        }

        i = i.wrapping_add(1);
    }
}
