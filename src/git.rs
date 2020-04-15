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

use std::{ffi::CStr, marker::PhantomData, mem::MaybeUninit, ptr, slice, str};

use libc::{c_char, c_int, c_void};
use libgit2_sys::{
    git_buf, git_commit, git_object, git_oid, git_reference, git_repository, GIT_OBJECT_ANY,
    GIT_OBJECT_COMMIT, GIT_REPOSITORY_OPEN_FROM_ENV,
};

pub struct Repository(*mut git_repository);

impl Repository {
    pub fn open_from_env() -> Option<Repository> {
        unsafe { libgit2_sys::git_libgit2_init() };

        let mut repo = MaybeUninit::uninit();

        match unsafe {
            libgit2_sys::git_repository_open_ext(
                &mut *repo.as_mut_ptr(),
                ptr::null(),
                GIT_REPOSITORY_OPEN_FROM_ENV,
                ptr::null(),
            )
        } {
            0 => Some(Repository(unsafe { repo.assume_init() })),
            _ => None,
        }
    }

    pub fn head(&self) -> Option<Reference> {
        let mut head = MaybeUninit::uninit();

        match unsafe { libgit2_sys::git_repository_head(&mut *head.as_mut_ptr(), self.0) } {
            0 => Some(Reference(unsafe { head.assume_init() }, PhantomData)),
            _ => None,
        }
    }

    pub fn lookup_object(&self, oid: Oid) -> Option<Object> {
        let mut obj = MaybeUninit::uninit();

        match unsafe {
            libgit2_sys::git_object_lookup(&mut *obj.as_mut_ptr(), self.0, oid.0, GIT_OBJECT_ANY)
        } {
            0 => Some(Object(unsafe { obj.assume_init() }, PhantomData)),
            _ => None,
        }
    }
}

impl<'a> Repository {
    pub fn tags_pointing_to(&'a self, commit: &'a Commit) -> Option<Vec<&'a CStr>> {
        let mut payload = Payload {
            repo: self,
            commit,
            result: Vec::new(),
        };
        let payload_ptr = &mut payload as *mut _ as *mut c_void;

        if unsafe {
            libgit2_sys::git_tag_foreach(self.0, Some(Repository::tag_cb_entry), payload_ptr)
        } != 0
        {
            None
        } else {
            Some(payload.result)
        }
    }

    fn tag_cb(&'a self, commit: &'a Commit, name: &'a CStr, oid: Oid, result: &mut Vec<&'a CStr>) {
        if let Some(obj) = self.lookup_object(oid) {
            if let Some(target_commit) = obj.peel_to_commit() {
                if commit.id() == target_commit.id() {
                    result.push(name);
                }
            }
        }
    }

    extern "C" fn tag_cb_entry(
        name: *const c_char,
        oid: *mut git_oid,
        payload: *mut c_void,
    ) -> c_int {
        const OFFSET: isize = 10; // "refs/tags/".len()

        let name = unsafe { CStr::from_ptr(name.offset(OFFSET)) };
        let payload = unsafe { &mut *(payload as *mut Payload) };

        let oid = Oid(oid, PhantomData);
        payload
            .repo
            .tag_cb(payload.commit, name, oid, &mut payload.result);

        0
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        unsafe { libgit2_sys::git_repository_free(self.0) };
        unsafe { libgit2_sys::git_libgit2_shutdown() };
    }
}

#[repr(C)]
struct Payload<'a> {
    repo: &'a Repository,
    commit: &'a Commit<'a>,
    result: Vec<&'a CStr>,
}

pub struct Reference<'repo>(*mut git_reference, PhantomData<&'repo Repository>);

impl<'repo> Reference<'repo> {
    pub fn branch_name(&self) -> Option<&CStr> {
        let mut name = MaybeUninit::uninit();

        match unsafe { libgit2_sys::git_branch_name(&mut *name.as_mut_ptr(), self.0) } {
            0 => Some(unsafe { CStr::from_ptr(name.assume_init()) }),
            _ => None,
        }
    }

    pub fn peel_to_commit(&self) -> Option<Commit<'repo>> {
        let mut commit = MaybeUninit::uninit();

        match unsafe {
            libgit2_sys::git_reference_peel(&mut *commit.as_mut_ptr(), self.0, GIT_OBJECT_COMMIT)
        } {
            0 => Some(Commit(
                unsafe { commit.assume_init() } as *mut git_commit,
                PhantomData,
            )),
            _ => None,
        }
    }
}

impl<'repo> Drop for Reference<'repo> {
    fn drop(&mut self) {
        unsafe { libgit2_sys::git_reference_free(self.0) };
    }
}

pub struct Commit<'repo>(*mut git_commit, PhantomData<&'repo Repository>);

impl<'repo> Commit<'repo> {
    pub fn as_object(&self) -> Object<'repo> {
        Object(self.0 as *mut git_object, PhantomData)
    }

    pub fn id(&self) -> Oid<'repo> {
        Oid(unsafe { libgit2_sys::git_commit_id(self.0) }, PhantomData)
    }
}

impl<'repo> Drop for Commit<'repo> {
    fn drop(&mut self) {
        unsafe { libgit2_sys::git_commit_free(self.0) };
    }
}

pub struct Object<'repo>(*mut git_object, PhantomData<&'repo Repository>);

impl<'repo> Object<'repo> {
    pub fn peel_to_commit(&self) -> Option<Commit<'repo>> {
        let mut commit = MaybeUninit::uninit();

        match unsafe {
            libgit2_sys::git_object_peel(&mut *commit.as_mut_ptr(), self.0, GIT_OBJECT_COMMIT)
        } {
            0 => Some(Commit(
                unsafe { commit.assume_init() } as *mut git_commit,
                PhantomData,
            )),
            _ => None,
        }
    }

    pub fn short_id(&self) -> Option<String> {
        let mut buf = git_buf {
            ptr: ptr::null_mut(),
            asize: 0,
            size: 0,
        };

        match unsafe { libgit2_sys::git_object_short_id(&mut buf, self.0) } {
            0 => (),
            _ => return None,
        };

        let buf = Buf(buf);
        let owned = unsafe { str::from_utf8_unchecked(buf.as_slice()).to_string() };

        Some(owned)
    }
}

struct Buf(git_buf);

impl Buf {
    unsafe fn as_slice(&self) -> &[u8] {
        slice::from_raw_parts(self.0.ptr as *mut u8, self.0.size as usize)
    }
}

impl Drop for Buf {
    fn drop(&mut self) {
        unsafe { libgit2_sys::git_buf_dispose(&mut self.0) };
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Oid<'repo>(*const git_oid, PhantomData<&'repo Repository>);

impl<'repo> PartialEq for Oid<'repo> {
    fn eq(&self, other: &Oid) -> bool {
        unsafe { libgit2_sys::git_oid_equal(self.0, other.0) != 0 }
    }
}

impl<'repo> Eq for Oid<'repo> {}
