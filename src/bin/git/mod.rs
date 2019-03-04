use std::{ffi::CStr, mem, ptr};

use libc::{c_char, c_int, c_void, size_t};
use libgit2_sys::{
    git_commit, git_diff, git_diff_notify_cb, git_diff_options, git_oid, git_reference,
    git_repository, git_tree, GIT_DIFF_ENABLE_FAST_UNTRACKED_DIRS, GIT_DIFF_INCLUDE_UNMODIFIED,
    GIT_DIFF_INCLUDE_UNTRACKED, GIT_DIFF_SKIP_BINARY_CHECK, GIT_DIFF_UPDATE_INDEX, GIT_OBJ_COMMIT,
    GIT_OID_HEXSZ, GIT_REPOSITORY_OPEN_FROM_ENV,
};

pub struct Repository(*mut git_repository);

impl Repository {
    pub fn open_from_env() -> Result<Repository, c_int> {
        unsafe { libgit2_sys::git_libgit2_init() };

        let mut repo = ptr::null_mut();

        let errc = unsafe {
            libgit2_sys::git_repository_open_ext(
                &mut repo,
                ptr::null(),
                GIT_REPOSITORY_OPEN_FROM_ENV,
                ptr::null(),
            )
        };

        if errc != 0 {
            assert!(repo.is_null());

            Err(errc)
        } else {
            assert!(!repo.is_null());

            Ok(Repository(repo))
        }
    }

    pub fn head(&mut self) -> Result<Reference, c_int> {
        assert!(!self.0.is_null());

        let mut head_reference = ptr::null_mut();
        let errc = unsafe { libgit2_sys::git_repository_head(&mut head_reference, self.0) };

        if errc != 0 {
            assert!(head_reference.is_null());

            Err(errc)
        } else {
            assert!(!head_reference.is_null());

            Ok(Reference(head_reference))
        }
    }

    pub fn diff_tree_to_workdir_with_index(
        &mut self,
        old_tree: Option<&mut Tree>,
        opts: Option<&DiffOptions>,
    ) -> Result<Diff, c_int> {
        let old_tree_ptr: *mut git_tree = if let Some(t) = old_tree {
            t.0
        } else {
            ptr::null_mut()
        };

        let opts_ptr: *const git_diff_options = if let Some(o) = opts {
            &o.0
        } else {
            ptr::null()
        };

        let mut diff = ptr::null_mut();
        let errc = unsafe {
            libgit2_sys::git_diff_tree_to_workdir_with_index(
                &mut diff,
                self.0,
                old_tree_ptr,
                opts_ptr,
            )
        };

        if errc != 0 {
            assert!(diff.is_null());

            Err(errc)
        } else {
            assert!(!diff.is_null());

            Ok(Diff(diff))
        }
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        assert!(!self.0.is_null());

        unsafe { libgit2_sys::git_repository_free(self.0) };

        unsafe { libgit2_sys::git_libgit2_shutdown() };
    }
}

pub struct Reference(*mut git_reference);

impl Reference {
    pub fn peel_to_commit(&self) -> Result<Commit, c_int> {
        assert!(!self.0.is_null());

        let mut commit = ptr::null_mut();
        let errc = unsafe { libgit2_sys::git_reference_peel(&mut commit, self.0, GIT_OBJ_COMMIT) };

        if errc != 0 {
            assert!(commit.is_null());

            Err(errc)
        } else {
            assert!(!commit.is_null());

            Ok(Commit(commit as *mut git_commit))
        }
    }

    pub fn branch_name(&self) -> Result<&CStr, c_int> {
        assert!(!self.0.is_null());

        let mut name = ptr::null();
        let errc = unsafe { libgit2_sys::git_branch_name(&mut name, self.0) };

        if errc != 0 {
            assert!(name.is_null());

            Err(errc)
        } else {
            assert!(!name.is_null());

            Ok(unsafe { CStr::from_ptr(name) })
        }
    }
}

impl Drop for Reference {
    fn drop(&mut self) {
        assert!(!self.0.is_null());

        unsafe { libgit2_sys::git_reference_free(self.0) };
    }
}

pub struct DiffOptions(git_diff_options);

impl DiffOptions {
    pub fn new() -> DiffOptions {
        let mut options = unsafe { mem::zeroed() };
        let errc = unsafe { libgit2_sys::git_diff_init_options(&mut options, 1) };
        assert_eq!(errc, 0);

        DiffOptions(options)
    }

    pub fn include_untracked(mut self) -> DiffOptions {
        self.0.flags |= GIT_DIFF_INCLUDE_UNTRACKED as u32;

        self
    }

    pub fn skip_binary_check(mut self) -> DiffOptions {
        self.0.flags |= GIT_DIFF_SKIP_BINARY_CHECK as u32;

        self
    }

    pub fn enable_fast_untracked_dirs(mut self) -> DiffOptions {
        self.0.flags |= GIT_DIFF_ENABLE_FAST_UNTRACKED_DIRS as u32;

        self
    }

    pub fn set_notify_cb(mut self, cb: git_diff_notify_cb) -> DiffOptions {
        self.0.notify_cb = cb;

        self
    }

    pub fn set_payload(mut self, payload: *mut c_void) -> DiffOptions {
        self.0.payload = payload;

        self
    }
}

pub struct Diff(*mut git_diff);

impl Drop for Diff {
    fn drop(&mut self) {
        assert!(!self.0.is_null());

        unsafe { libgit2_sys::git_diff_free(self.0) };
    }
}

pub struct Commit(*mut git_commit);

impl Commit {
    pub fn id(&self) -> Oid {
        assert!(!self.0.is_null());

        let oid = unsafe { libgit2_sys::git_commit_id(self.0) };
        assert!(!oid.is_null());

        Oid(oid)
    }

    pub fn tree(&self) -> Result<Tree, c_int> {
        assert!(!self.0.is_null());

        let mut tree = ptr::null_mut();
        let errc = unsafe { libgit2_sys::git_commit_tree(&mut tree, self.0) };

        if errc != 0 {
            assert!(tree.is_null());

            Err(errc)
        } else {
            assert!(!tree.is_null());

            Ok(Tree(tree))
        }
    }
}

impl Drop for Commit {
    fn drop(&mut self) {
        assert!(!self.0.is_null());

        unsafe { libgit2_sys::git_commit_free(self.0) };
    }
}

pub struct Tree(*mut git_tree);

impl Drop for Tree {
    fn drop(&mut self) {
        assert!(!self.0.is_null());

        unsafe { libgit2_sys::git_tree_free(self.0) };
    }
}

pub struct Oid(*const git_oid);

impl ToString for Oid {
    fn to_string(&self) -> String {
        assert!(!self.0.is_null());

        let mut hex_bytes = Vec::with_capacity(GIT_OID_HEXSZ as usize + 1);
        unsafe {
            libgit2_sys::git_oid_tostr(
                hex_bytes.as_mut_ptr() as *mut c_char,
                hex_bytes.capacity() as size_t,
                self.0,
            )
        };
        unsafe { hex_bytes.set_len(GIT_OID_HEXSZ as usize) };

        unsafe { String::from_utf8_unchecked(hex_bytes) }
    }
}
