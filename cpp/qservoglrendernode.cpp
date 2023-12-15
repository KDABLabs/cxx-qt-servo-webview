
#include "qservoglrendernode.h"

#include <QQuickItem>
#include <QDebug>

#include "cxx-qt-gen/servowebview.cxxqt.h"

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

    if (m_swapChain.has_value())
    {
        qWarning() << Q_FUNC_INFO << "take surface as texture";
        auto texture = (*m_swapChain)->take_surface_as_texture();
        qWarning() << Q_FUNC_INFO << "got surface!";

        // TODO: render texture!

        // (*m_swapChain)->recycle_surface(std::move(surface));
    }

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

    if (!m_swapChain.has_value())
    {
        if (auto webview = dynamic_cast<ServoWebView*>(item)) {
            qWarning() << Q_FUNC_INFO << "found webview";
            m_swapChain = std::move(webview->swapChain());
        } else {
            qWarning() << Q_FUNC_INFO << "missing webview";
        }
    }
}

#endif // opengl