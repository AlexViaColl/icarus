use std::ffi::c_void;
//use std::ptr;

pub type GLenum = u32;
pub type GLbitfield = u32;
pub type GLuint = u32;
pub type GLint = i32;
pub type GLsizei = i32;
pub type GLboolean = u8;
pub type GLbyte = i8;
pub type GLshort = i16;
pub type GLubyte = u8;
pub type GLushort = u16;
pub type GLulong = u64;
pub type GLfloat = f32;
pub type GLclampf = f32;
pub type GLdouble = f64;
pub type GLclampd = f64;
pub type GLvoid = c_void;
pub type GLchar = i8;

pub type GLsizeiptr = i64;
pub type GLintptr = i64;

pub const GL_DEPTH_BUFFER_BIT: GLbitfield = 0x00000100;
pub const GL_COLOR_BUFFER_BIT: GLbitfield = 0x00004000;

pub const GL_DEPTH_TEST: GLenum = 0x0B71;

pub const GL_LESS: GLenum = 0x0201;

pub const GL_VENDOR: GLenum = 0x1F00;
pub const GL_RENDERER: GLenum = 0x1F01;
pub const GL_VERSION: GLenum = 0x1F02;
pub const GL_EXTENSIONS: GLenum = 0x1F03;

pub const GL_SHADING_LANGUAGE_VERSION: GLenum = 0x8B8C;

pub const GL_LINES: GLenum = 0x0001;
pub const GL_LINE_LOOP: GLenum = 0x0002;
pub const GL_LINE_STRIP: GLenum = 0x0003;
pub const GL_TRIANGLES: GLenum = 0x0004;
pub const GL_TRIANGLE_STRIP: GLenum = 0x0005;
pub const GL_TRIANGLE_FAN: GLenum = 0x0006;
pub const GL_QUADS: GLenum = 0x0007;
pub const GL_QUAD_STRIP: GLenum = 0x0008;
pub const GL_POLYGON: GLenum = 0x0009;

#[link(name = "GL")]
extern "C" {
    // 1.1
    pub fn glBegin(mode: GLenum);
    pub fn glEnd();

    pub fn glClear(mask: GLbitfield);
    pub fn glClearColor(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf);
    pub fn glClearDepth(depth: GLclampd);
    pub fn glClearStencil(s: GLint);

    pub fn glColor3b(red: GLbyte, green: GLbyte, blue: GLbyte);
    pub fn glColor3f(red: GLfloat, green: GLfloat, blue: GLfloat);
    pub fn glColor4b(red: GLbyte, green: GLbyte, blue: GLbyte, alpha: GLbyte);
    pub fn glColor4f(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat);

    pub fn glStencilFunc(func: GLenum, ref_: GLint, mask: GLuint);
    pub fn glStencilMask(mask: GLuint);
    pub fn glStencilOp(fail: GLenum, zfail: GLenum, zpass: GLenum);

    pub fn glDepthFunc(func: GLenum);
    pub fn glDepthMask(flag: GLboolean);
    pub fn glDisable(cap: GLenum);
    pub fn glEnable(cap: GLenum);

    pub fn glBlendFunc(sfactor: GLenum, dfactor: GLenum);
    pub fn glCullFace(mode: GLenum);
    pub fn glFrontFace(mode: GLenum);

    pub fn glFinish();
    pub fn glFlush();

    pub fn glGetIntegerv(pname: GLenum, params: *mut GLint);
    pub fn glGetError() -> GLenum;

    pub fn glGetString(name: GLenum) -> *const GLubyte;

    pub fn glHint(target: GLenum, mode: GLenum);

    pub fn glNormal3f(nx: GLfloat, ny: GLfloat, nz: GLfloat);

    pub fn glTexCoord2f(s: GLfloat, t: GLfloat);

    pub fn glVertex2f(x: GLfloat, y: GLfloat);
    pub fn glVertex2fv(v: *const GLfloat);
    pub fn glVertex3f(x: GLfloat, y: GLfloat, z: GLfloat);
    pub fn glVertex3fv(v: *const GLfloat);
    pub fn glVertex4f(x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat);
    pub fn glVertex4fv(v: *const GLfloat);
    pub fn glVertexPointer(size: GLint, ttype: GLenum, stride: GLsizei, pointer: *const c_void);

    pub fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei);

    pub fn glGenTextures(n: GLsizei, textures: *mut GLuint);
    pub fn glDeleteTexture(n: GLsizei, textures: *const GLuint);
    pub fn glBindTexture(target: GLenum, texture: GLuint);
    pub fn glTexImage2D(
        target: GLenum,
        level: GLint,
        internalFormat: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        ttype: GLenum,
        pixels: *const GLvoid,
    );

    pub fn glDrawArrays(mode: GLenum, first: GLint, count: GLsizei);
    pub fn glDrawElements(mode: GLenum, count: GLsizei, ttype: GLenum, indices: *const GLvoid);

    pub fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint);
    pub fn glTexParameterfv(target: GLenum, pname: GLenum, params: *const GLfloat);

    pub fn glReadBuffer(mode: GLenum);

    // 1.2
    // pub fn glCopyTexSubImage3D(...);
    // pub fn glDrawRangeElements(...);
    // pub fn glTexImage3D(...);
    // pub fn glTexSubImage3D(...);

    // 1.3
    pub fn glActiveTexture(texture: GLenum);
    // ...
    // pub fn glSampleCoverage(value: GLclampf, invert: GLboolean);

    // 1.4
    // pub fn glBlendColor(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf);
    // ...
    // pub fn glWindowPos3sv(p: *const GLshort);

    // 1.5
    pub fn glBindBuffer(target: GLenum, buffer: GLuint);
    pub fn glDeleteBuffers(n: GLsizei, buffers: *const GLuint);
    pub fn glGenBuffers(n: GLsizei, buffers: *mut GLuint);
    pub fn glBufferData(target: GLenum, size: GLsizeiptr, data: *const c_void, usage: GLenum);
    pub fn glBufferSubData(target: GLenum, offset: GLintptr, size: GLsizeiptr, data: *const c_void);
    pub fn glMapBuffer(target: GLenum, access: GLenum) -> *mut c_void;
    pub fn glUnmapBuffer(target: GLenum) -> GLboolean;
    // ...

    // 2.0
    // pub fn glAttachShader(program: GLuint, shader: GLuint);
    pub fn glAttachShader(program: GLuint, shader: GLuint);
    pub fn glCompileShader(shader: GLuint);
    pub fn glCreateProgram() -> GLuint;
    pub fn glCreateShader(ttype: GLenum) -> GLuint;
    pub fn glDeleteProgram(program: GLuint);
    pub fn glDeleteShader(shader: GLuint);
    pub fn glDetachShader(program: GLuint, shader: GLuint);
    pub fn glDisableVertexAttribArray(index: GLuint);
    pub fn glEnableVertexAttribArray(index: GLuint);

    pub fn glGetProgramiv(program: GLuint, pname: GLenum, params: *mut GLint);
    pub fn glGetProgramInfoLog(program: GLuint, bufSize: GLsizei, length: *mut GLsizei, infoLog: *mut GLchar);
    pub fn glGetShaderiv(shader: GLuint, pname: GLenum, params: *mut GLint);
    pub fn glGetShaderInfoLog(shader: GLuint, bufSize: GLsizei, length: *mut GLsizei, infoLog: *mut GLchar);
    pub fn glGetShaderSource(shader: GLuint, bufSize: GLsizei, length: *mut GLsizei, source: *mut GLchar);
    pub fn glGetUniformLocation(program: GLuint, name: *const GLchar) -> GLint;

    pub fn glLinkProgram(program: GLuint);
    pub fn glShaderSource(shader: GLuint, count: GLsizei, string: *const *const GLchar, length: *const GLint);
    pub fn glUseProgram(program: GLuint);
    pub fn glUniform1f(location: GLint, v0: GLfloat);
    pub fn glUniform2f(location: GLint, v0: GLfloat, v1: GLfloat);
    pub fn glUniform3f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat);
    pub fn glUniform4f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat, v3: GLfloat);
    pub fn glUniform1i(location: GLint, v0: GLint);
    pub fn glUniform2i(location: GLint, v0: GLint, v1: GLint);
    pub fn glUniform3i(location: GLint, v0: GLint, v1: GLint, v2: GLint);
    pub fn glUniform4i(location: GLint, v0: GLint, v1: GLint, v2: GLint, v3: GLint);
    // ...
    pub fn glUniformMatrix4fv(location: GLint, count: GLsizei, transpose: GLboolean, value: *const GLfloat);

    pub fn glGenFramebuffers(n: GLsizei, framebuffers: *mut GLuint);
    pub fn glDeleteFramebuffers(n: GLsizei, framebuffers: *const GLuint);
    pub fn glBindFramebuffer(target: GLenum, framebuffer: GLuint);
    pub fn glCheckFramebufferStatus(target: GLenum) -> GLenum;
    pub fn glFramebufferTexture2D(target: GLenum, attachment: GLenum, textarget: GLenum, texture: GLuint, level: GLint);
    pub fn glFramebufferRenderbuffer(
        target: GLenum,
        attachment: GLenum,
        renderbuffertarget: GLenum,
        renderbuffer: GLuint,
    );

    pub fn glGenRenderbuffers(n: GLsizei, renderbuffers: *mut GLuint);
    pub fn glDeleteRenderbuffers(n: GLsizei, renderbuffers: *const GLuint);
    pub fn glBindRenderbuffer(target: GLenum, renderbuffer: GLuint);
    pub fn glRenderbufferStorage(target: GLenum, internalformat: GLenum, width: GLsizei, height: GLsizei);

    // ...
    pub fn glVertexAttribPointer(
        index: GLuint,
        size: GLint,
        ttype: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        pointer: *const c_void,
    );

    // 2.1
    // pub fn glUniformMatrix2x3fv(...);
    // ...
    // pub fn glUniformMatrix4x3fv(...);

    // 3.0
    // pub fn glBeginConditionalRender(id: GLuint, mode: GLenum);
    // ...
    // pub fn glVertexAttribPointer(index: GLuint, size: GLint, ttype: GLenum, stride: GLsizei, pointer: *const c_void);
    pub fn glBindVertexArray(array: GLuint);
    pub fn glDeleteVertexArrays(n: GLsizei, arrays: *const GLuint);
    pub fn glGenVertexArrays(n: GLsizei, arrays: *mut GLuint);

    pub fn glGenerateMipmap(target: GLenum);
    pub fn glBlitFramebuffer(
        srcX0: GLint,
        srcY0: GLint,
        srcX1: GLint,
        srcY1: GLint,
        dstX0: GLint,
        dstY0: GLint,
        dstX1: GLint,
        dstY1: GLint,
        mask: GLbitfield,
        filter: GLenum,
    );
    pub fn glRenderbufferStorageMultisample(
        target: GLenum,
        samples: GLsizei,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei,
    );

    pub fn glBindBufferBase(target: GLenum, index: GLuint, buffer: GLuint);
    pub fn glBindBufferRange(target: GLenum, index: GLuint, buffer: GLuint, offset: GLintptr, size: GLsizeiptr);

    // 3.1
    pub fn glCopyBufferSubData(
        readTarget: GLenum,
        writeTarget: GLenum,
        readOffset: GLintptr,
        writeOffset: GLintptr,
        size: GLsizeiptr,
    );
    pub fn glGetUniformBlockIndex(program: GLuint, uniformBlockName: *const GLchar) -> GLuint;
    pub fn glUniformBlockBinding(program: GLuint, uniformBlockIndex: GLuint, uniformBlockBinding: GLuint);
    pub fn glDrawArraysInstanced(mode: GLenum, first: GLint, count: GLsizei, instancecount: GLsizei);
    pub fn glDrawElementsInstanced(
        mode: GLenum,
        count: GLsizei,
        ttype: GLenum,
        indices: *const c_void,
        instancecount: GLsizei,
    );
    // ...

    // 3.2
    pub fn glTexImage2DMultisample(
        target: GLenum,
        samples: GLsizei,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei,
        fixedsamplelocation: GLboolean,
    );
    // ...

    // 3.3
    pub fn glVertexAttribDivisor(index: GLuint, divisor: GLuint);

    // 4.0
    // ...

    // 4.3
    pub fn glDebugMessageCallback(
        callback: extern "C" fn(GLenum, GLenum, GLuint, GLenum, GLsizei, *const GLchar, *const c_void),
        userParam: *const c_void,
    );
    pub fn glDebugMessageControl(
        source: GLenum,
        ttype: GLenum,
        id: GLuint,
        severity: GLenum,
        length: GLsizei,
        buf: *const GLchar,
    );
    pub fn glDebugMessageInsert(
        source: GLenum,
        ttype: GLenum,
        id: GLuint,
        severity: GLenum,
        length: GLsizei,
        buf: *const GLchar,
    );

    // 4.5
    // pub fn glGetGraphicsResetStatus() -> GLenum;
    // pub fn glGetnCompressedTexImage(target: GLenu, lod: GLint, bufSize: GLsizei, pixels: *mut c_void);
    // pub fn glGetnTexImage(tex: GLenum, level: GLint, format: GLenum, ttype: GLenum, bufSize: GLsizei, pixels: *mut GLvoid);
    // pub fn glGetnUniformdv(program: GLuint, location: GLint, bufSize: GLsizei, params: *mut GLdouble);

    // 4.6
    // pub fn glMultiDrawArraysIndirectCount(mode: GLenum, indirect: *const GLvoid, drawcount: GLintptr, maxdrawcount: GLsizei, stride: GLsizei);
    // pub fn glMultiDrawElementsIndirectCount(mode: GLenum, ttype: GLenum indirect: *const GLvoid, drawcount: GLintptr, maxdrawcount: GLsizei, stride: GLsizei);
    // pub fn glSpecializeShader(shader: GLuint, pEntryPoint: *const GLchar, numSpecializationConstants: GLuint, pConstantIndex: *const GLuint, pConstantValu: *const GLuint);

    // Extensions
}

pub const GL_MAJOR_VERSION: GLenum = 0x821B;
pub const GL_MINOR_VERSION: GLenum = 0x821C;
pub const GL_NUM_EXTENSIONS: GLenum = 0x821D;
pub const GL_CONTEXT_FLAGS: GLenum = 0x821E;
pub const GL_CONTEXT_PROFILE_MASK: GLenum = 0x9126; // GL_CONTEXT_CORE_PROFILE_BIT or GL_CONTEXT_COMPATIBILITY_PROFILE_BIT

pub const GL_CONTEXT_CORE_PROFILE_BIT: GLint = 0x00000001;
pub const GL_CONTEXT_COMPATIBILITY_PROFILE_BIT: GLint = 0x00000002;

#[cfg(test)]
mod tests {
    //use super::*;
    //use crate::string_util::cstr_to_string;

    #[test]
    #[ignore]
    fn it_works() {
        //unsafe {
        //    println!("glGetError: {}", glGetError());
        //    println!("{:?}", glGetString(GL_VENDOR));
        //    println!("glGetError: {}", glGetError());
        //    println!("{}", cstr_to_string(glGetString(GL_VENDOR) as *const i8));
        //    glClearColor(1.0, 1.0, 1.0, 1.0);
        //}
    }
}
