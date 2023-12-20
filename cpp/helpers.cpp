#include "helpers.h"

#include <QOpenGLFramebufferObject>
#include <QOpenGLContext>
#include <QOpenGLFunctions>


QQuickFramebufferObjectRendererWithQObject::QQuickFramebufferObjectRendererWithQObject()
    : QQuickFramebufferObjectRenderer()
    , QObject(nullptr)
{
}

QQuickFramebufferObjectRendererWithQObject::~QQuickFramebufferObjectRendererWithQObject()
{
}

void
blitFramebuffer(QOpenGLFramebufferObject* target, QOpenGLFramebufferObject* source)
{
    QOpenGLFramebufferObject::blitFramebuffer(target, source);
}

QOpenGLFramebufferObject*
fboFromTexture(unsigned int texture_id, unsigned int texture_target, QSize size)
{
    QOpenGLFunctions *f = QOpenGLContext::currentContext()->functions();

    auto fbo = new QOpenGLFramebufferObject(size);
    f->glFramebufferTexture2D(GL_READ_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, texture_target, texture_id, 0);
    Q_ASSERT(fbo->isValid());
    return fbo;
}
