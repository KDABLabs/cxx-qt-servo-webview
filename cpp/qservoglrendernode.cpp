
#include "qservoglrendernode.h"

#include <QQuickItem>

#if QT_CONFIG(opengl)

QServoGLRenderNode::~QServoGLRenderNode()
{
    releaseResources();
}

void QServoGLRenderNode::releaseResources()
{

}

void QServoGLRenderNode::init()
{

}

void QServoGLRenderNode::render(const RenderState *state)
{
    Q_UNUSED(state);

    // TODO: at this point do we render the WebRender context to the Qt context or something?

    // const auto* context = QOpenGLContext::currentContext();

    // TODO: here we need to tell webrender to render
}

QSGRenderNode::StateFlags QServoGLRenderNode::changedStates() const
{
    return BlendState | ScissorState | StencilState | DepthState;
}

QSGRenderNode::RenderingFlags QServoGLRenderNode::flags() const
{
    return BoundedRectRendering | DepthAwareRendering;
}

QRectF QServoGLRenderNode::rect() const
{
    return QRect(0, 0, m_width, m_height);
}

void QServoGLRenderNode::sync(QQuickItem *item)
{
    m_width = item->width();
    m_height = item->height();
}

#endif // opengl