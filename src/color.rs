// Copyright (C) 2020 Gregory Meyer
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::fmt::{self, Display, Formatter};

pub trait Color: Display {
    fn red(&self) -> Red<Self> {
        Red { t: self }
    }

    fn green(&self) -> Green<Self> {
        Green { t: self }
    }
}

impl<T: Display> Color for T {}

pub struct Red<'a, T: Display + ?Sized> {
    t: &'a T,
}

impl<'a, T: Display + ?Sized> Display for Red<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "\x1b[31m{}\x1b[0m", self.t)
    }
}

pub struct Green<'a, T: Display + ?Sized> {
    t: &'a T,
}

impl<'a, T: Display + ?Sized> Display for Green<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "\x1b[32m{}\x1b[0m", self.t)
    }
}
